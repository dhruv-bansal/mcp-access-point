docker run --name mcp-access-point --rm \
  -p 8080:8080 \
  -e port=8080 \
  -v /Users/dhruvbansal/Documents/code/mcp-config/config1/config.yaml:/app/config/config.yaml \
  liangshihua/mcp-access-point:latest