Extractors (Headers, params, query) & Custom types

```
# With body
curl -X POST http://127.0.0.1:3000/complex/1?name=ahmer \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}'

# Without body (still works!)
curl -X POST http://127.0.0.1:3000/complex/1?name=ahmer

```