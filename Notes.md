## Request at upstream via Cursor

### Create user by username

```json
{
  "origin": "remote",
  "type": "request",
  "correlation": "e1cab96bba4807c6",
  "protocol": "HTTP/1.1",
  "remote": "127.0.0.1",
  "method": "POST",
  "uri": "http://localhost:8080/user",
  "host": "localhost",
  "path": "/user",
  "scheme": "http",
  "port": "8080",
  "headers": {
    "accept": ["application/json"],
    "accept-encoding": ["gzip, deflate"],
    "accept-language": ["*"],
    "connection": ["keep-alive"],
    "content-length": ["238"],
    "content-type": ["application/json"],
    "host": ["localhost:8080"],
    "mcp_tenant_id": ["service-1"],
    "sec-fetch-mode": ["cors"],
    "user-agent": ["node"]
  },
  "body": {
    "method": "tools/call",
    "params": {
      "name": "createUser",
      "arguments": {
        "username": "test",
        "firstName": "Test",
        "lastName": "User",
        "email": "testuser@example.com",
        "password": "changeme123",
        "phone": "0000000000",
        "userStatus": 1
      }
    },
    "jsonrpc": "2.0",
    "id": 6
  }
}
```

#### After code change

```json
{
  "origin": "remote",
  "type": "request",
  "correlation": "9afa6e74339dfaaa",
  "protocol": "HTTP/1.1",
  "remote": "127.0.0.1",
  "method": "POST",
  "uri": "http://localhost:8080/user",
  "host": "localhost",
  "path": "/user",
  "scheme": "http",
  "port": "8080",
  "headers": {
    "accept": ["application/json"],
    "accept-encoding": ["gzip, deflate"],
    "accept-language": ["*"],
    "connection": ["keep-alive"],
    "content-length": ["152"],
    "content-type": ["application/json"],
    "host": ["localhost:8080"],
    "mcp_tenant_id": ["service-1"],
    "sec-fetch-mode": ["cors"],
    "user-agent": ["node"]
  },
  "body": {
    "email": "user5678@example.com",
    "firstName": "User",
    "lastName": "Test",
    "password": "changeme123",
    "phone": "1111111111",
    "userStatus": 1,
    "username": "user5678"
  }
}
```

## Get user by username

```json
{
  "origin": "remote",
  "type": "request",
  "correlation": "a8903df0070d983d",
  "protocol": "HTTP/1.1",
  "remote": "127.0.0.1",
  "method": "GET",
  "uri": "http://localhost:8080/user/theUser",
  "host": "localhost",
  "path": "/user/theUser",
  "scheme": "http",
  "port": "8080",
  "headers": {
    "accept": ["application/json"],
    "accept-encoding": ["gzip, deflate"],
    "accept-language": ["*"],
    "connection": ["keep-alive"],
    "content-length": ["115"],
    "content-type": ["application/json"],
    "host": ["localhost:8080"],
    "mcp_tenant_id": ["service-1"],
    "sec-fetch-mode": ["cors"],
    "user-agent": ["node"]
  },
  "body": {
    "method": "tools/call",
    "params": {
      "name": "getUserByName",
      "arguments": {
        "username": "theUser"
      }
    },
    "jsonrpc": "2.0",
    "id": 7
  }
}
```

### Delete user by username

```json
{
  "origin": "remote",
  "type": "request",
  "correlation": "8f03686b59e9a371",
  "protocol": "HTTP/1.1",
  "remote": "127.0.0.1",
  "method": "DELETE",
  "uri": "http://localhost:8080/user/theUser",
  "host": "localhost",
  "path": "/user/theUser",
  "scheme": "http",
  "port": "8080",
  "headers": {
    "accept": ["application/json"],
    "accept-encoding": ["gzip, deflate"],
    "accept-language": ["*"],
    "connection": ["keep-alive"],
    "content-length": ["112"],
    "content-type": ["application/json"],
    "host": ["localhost:8080"],
    "mcp_tenant_id": ["service-1"],
    "sec-fetch-mode": ["cors"],
    "user-agent": ["node"]
  },
  "body": {
    "method": "tools/call",
    "params": {
      "name": "deleteUser",
      "arguments": {
        "username": "theUser"
      }
    },
    "jsonrpc": "2.0",
    "id": 8
  }
}
```


Notes latest

Complete RPC json body is beign passed
< Body: {"method":"tools/call","params":{"name":"updatePet","arguments":{"body":{"id":1,"name":"UpdatedPetName","category":{"id":1,"name":"Dogs"},"photoUrls":["http://example.com/photo1.jpg"],"tags":[{"id":1,"name":"tag1"}],"status":"available"},"petId":1}},"jsonrpc":"2.0","id":6}