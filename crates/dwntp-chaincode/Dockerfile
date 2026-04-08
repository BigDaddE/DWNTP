FROM alpine:latest

WORKDIR /app

# Copy the pre-built binary from the host
COPY dwntp-chaincode .

# Expose port 9999 to the outside world
EXPOSE 9999

# Command to run the executable
CMD ["./dwntp-chaincode"]
