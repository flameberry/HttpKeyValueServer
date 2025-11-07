# HTTP based Key Value Server (CS744 course project)

## Running the server
``` bash
cd server
cargo run
```

## Sending a request
### Put Request 
``` bash
curl -X PUT http://127.0.0.1:3000/kv/<key> \
     -H "Content-Type: application/json" \
     -d '{"value": "new_value"}'
```

Response:
```json
{
  "id": "uuid",
  "key": "sample_key",
  "value": "new_value"
}

```

### Get Request
``` bash
curl -X GET http://127.0.0.1:3000/kv/<key>
```

Response:
``` json
{
  "id": "uuid",
  "key": "sample_key",
  "value": "sample_value"
}
```

### Delete Request
``` bash
curl -X DELETE http://127.0.0.1:3000/kv/<key>
```

Response:
``` 
Deleted key: '<key>' successfully.
```
