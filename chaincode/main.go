package main

import (
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"os"

	"github.com/hyperledger/fabric-chaincode-go/shim"
	"github.com/hyperledger/fabric-contract-api-go/contractapi"
)

// SmartContract provides functions for managing DWNTP events
type SmartContract struct {
	contractapi.Contract
}

// RtuControlEvent represents a control action in the smart grid network.
type RtuControlEvent struct {
	ID               string `json:"id"`
	SourceMtu        string `json:"source_mtu"`
	RtuID            string `json:"rtu_id"`
	EventName        string `json:"event_name"`
	EventDescription string `json:"event_description"`
	EventTimestamp   uint64 `json:"event_timestamp"`
	OnChainTimestamp uint64 `json:"on_chain_timestamp,omitempty"`
}

// CanonicalEvent is used to generate the deterministic ID matching the Rust implementation.
type CanonicalEvent struct {
	SourceMtu      string `json:"source_mtu"`
	RtuID          string `json:"rtu_id"`
	EventName      string `json:"event_name"`
	EventTimestamp uint64 `json:"event_timestamp"`
}

const eventKeyPrefix = "event_"
const eventDocType = "event"

// LogEvent creates a new RTU control event and stores it on the ledger.
func (s *SmartContract) LogEvent(ctx contractapi.TransactionContextInterface, rtuId string, eventName string, eventDescription string, eventTimestamp uint64) (string, error) {
	// Validate inputs
	if rtuId == "" {
		return "", fmt.Errorf("missing rtu_id")
	}
	if eventName == "" {
		return "", fmt.Errorf("missing event_name")
	}
	if eventDescription == "" {
		return "", fmt.Errorf("missing event_description")
	}

	cert, err := ctx.GetClientIdentity().GetX509Certificate()
	if err != nil {
		return "", fmt.Errorf("failed to get client certificate: %v", err)
	}
	if cert == nil {
		return "", fmt.Errorf("client identity is not backed by an x509 certificate")
	}

	actualSourceMtu := cert.Subject.CommonName
	sourceMtuBase64 := base64.StdEncoding.EncodeToString([]byte(actualSourceMtu))

	// Generate deterministic ID
	canonical := CanonicalEvent{
		SourceMtu:      sourceMtuBase64,
		RtuID:          rtuId,
		EventName:      eventName,
		EventTimestamp: eventTimestamp,
	}

	canonicalBytes, err := json.Marshal(canonical)
	if err != nil {
		return "", fmt.Errorf("failed to marshal canonical event: %v", err)
	}

	hash := sha256.Sum256(canonicalBytes)
	id := hex.EncodeToString(hash[:])

	// Check if event already exists
	key := eventKeyPrefix + id
	existing, err := ctx.GetStub().GetState(key)
	if err != nil {
		return "", fmt.Errorf("failed to read from world state: %v", err)
	}
	if existing != nil {
		return "", fmt.Errorf("event already exists with id: %s", id)
	}

	// Get transaction timestamp for on-chain timestamp
	txTimestamp, err := ctx.GetStub().GetTxTimestamp()
	var onChainTs uint64
	if err == nil && txTimestamp != nil {
		onChainTs = uint64(txTimestamp.Seconds)*1000 + uint64(txTimestamp.Nanos)/1000000
	}

	event := RtuControlEvent{
		ID:               id,
		SourceMtu:        sourceMtuBase64,
		RtuID:            rtuId,
		EventName:        eventName,
		EventDescription: eventDescription,
		EventTimestamp:   eventTimestamp,
		OnChainTimestamp: onChainTs,
	}

	eventJSON, err := json.Marshal(event)
	if err != nil {
		return "", fmt.Errorf("failed to marshal event: %v", err)
	}

	// Put to state (Primary Key for direct queries)
	err = ctx.GetStub().PutState(key, eventJSON)
	if err != nil {
		return "", fmt.Errorf("failed to put state: %v", err)
	}

	// Create composite key for chronological sorting (Index)
	// Format: event~timestamp~id
	timestampStr := fmt.Sprintf("%020d", eventTimestamp)
	indexName := "timestamp~id"
	compositeKey, err := ctx.GetStub().CreateCompositeKey(indexName, []string{eventDocType, timestampStr, id})
	if err != nil {
		return "", fmt.Errorf("failed to create composite key: %v", err)
	}

	// Save the composite key index with an empty value
	value := []byte{0x00}
	err = ctx.GetStub().PutState(compositeKey, value)
	if err != nil {
		return "", fmt.Errorf("failed to put composite key state: %v", err)
	}

	return id, nil
}

// QueryEvent retrieves an event from the ledger by its ID.
func (s *SmartContract) QueryEvent(ctx contractapi.TransactionContextInterface, id string) (*RtuControlEvent, error) {
	key := eventKeyPrefix + id
	eventJSON, err := ctx.GetStub().GetState(key)
	if err != nil {
		return nil, fmt.Errorf("failed to read from world state: %v", err)
	}
	if eventJSON == nil {
		return nil, fmt.Errorf("the event %s does not exist", id)
	}

	var event RtuControlEvent
	err = json.Unmarshal(eventJSON, &event)
	if err != nil {
		return nil, fmt.Errorf("failed to unmarshal event JSON: %v", err)
	}

	return &event, nil
}

// GetAllEvents retrieves all events stored on the ledger, ordered chronologically by event_timestamp.
func (s *SmartContract) GetAllEvents(ctx contractapi.TransactionContextInterface) ([]*RtuControlEvent, error) {
	// Query the composite key index to get chronologically ordered events
	indexName := "timestamp~id"
	resultsIterator, err := ctx.GetStub().GetStateByPartialCompositeKey(indexName, []string{eventDocType})
	if err != nil {
		return nil, fmt.Errorf("failed to get state by partial composite key: %v", err)
	}
	defer resultsIterator.Close()

	var events []*RtuControlEvent
	for resultsIterator.HasNext() {
		queryResponse, err := resultsIterator.Next()
		if err != nil {
			return nil, fmt.Errorf("failed to iterate results: %v", err)
		}

		// Parse the composite key to get the event ID
		_, compositeKeyParts, err := ctx.GetStub().SplitCompositeKey(queryResponse.Key)
		if err != nil {
			return nil, fmt.Errorf("failed to split composite key: %v", err)
		}

		// Parts: [eventDocType, timestamp, id]
		if len(compositeKeyParts) < 3 {
			continue // skip invalid keys
		}
		id := compositeKeyParts[2]

		// Fetch the actual event data using the ID
		eventBytes, err := ctx.GetStub().GetState(eventKeyPrefix + id)
		if err != nil {
			return nil, fmt.Errorf("failed to get event data for id %s: %v", id, err)
		}
		if eventBytes == nil {
			continue // skip if event data is missing
		}

		var event RtuControlEvent
		err = json.Unmarshal(eventBytes, &event)
		if err != nil {
			return nil, fmt.Errorf("failed to unmarshal event JSON: %v", err)
		}
		events = append(events, &event)
	}

	// If no events found, return empty array instead of null
	if events == nil {
		events = make([]*RtuControlEvent, 0)
	}

	return events, nil
}

func main() {
	chaincode, err := contractapi.NewChaincode(&SmartContract{})
	if err != nil {
		log.Panicf("Error creating dwntp chaincode: %v", err)
	}

	serverAddress := os.Getenv("CHAINCODE_SERVER_ADDRESS")
	chaincodeID := os.Getenv("CHAINCODE_ID")

	if serverAddress != "" && chaincodeID != "" {
		log.Printf("Starting chaincode server on %s with ID %s", serverAddress, chaincodeID)
		chaincode.Info.Title = "DWNTP External Chaincode"
		chaincode.Info.Version = "1.0"

		server := &shim.ChaincodeServer{
			CCID:    chaincodeID,
			Address: serverAddress,
			CC:      chaincode,
			TLSProps: shim.TLSProperties{
				Disabled: true,
			},
		}

		if err := server.Start(); err != nil {
			log.Panicf("Error starting dwntp chaincode server: %v", err)
		}
	} else {
		log.Printf("Starting chaincode as local process (dev mode)")
		if err := chaincode.Start(); err != nil {
			log.Panicf("Error starting dwntp chaincode: %v", err)
		}
	}
}
