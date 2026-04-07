fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "fabric-protos/common/policies.proto",
        "fabric-protos/peer/chaincode_shim.proto",
        "fabric-protos/peer/chaincode_event.proto",
        "fabric-protos/peer/proposal.proto",
        "fabric-protos/peer/proposal_response.proto",
        "fabric-protos/ledger/queryresult/kv_query_result.proto",
    ];

    let includes = &["fabric-protos"];

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(proto_files, includes)?;

    // Recompile if the proto files change
    println!("cargo:rerun-if-changed=fabric-protos");

    Ok(())
}
