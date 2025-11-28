#!/bin/bash
URL="http://127.0.0.1:8080"
PASS=0 FAIL=0

test() {
    local name="$1"
    local method="$2"
    local path="$3"
    local expected="$4"
    local data="$5"
    local headers="$6"

    echo -n "$name... "
    
    if [ -n "$data" ]; then
        status=$(curl -sw "\n%{http_code}" -X "$method" -d "$data" $headers "$URL$path" 2>/dev/null | tail -1)
    else
        status=$(curl -sw "\n%{http_code}" -X "$method" $headers "$URL$path" 2>/dev/null | tail -1)
    fi

    if [ "$status" = "$expected" ]; then
        echo "✓" && ((PASS++))
    else
        echo "✗" && ((FAIL++))
    fi
}

echo "Testing rawhttp handlers"
test "GET /" GET "/" 200
test "GET /status" GET /status 200
test "GET /query" GET /query 200
test "GET /query?message=test" GET "/query?message=test" 200
test "GET /invalid" GET /invalid 404
test "POST /echo" POST /echo 200 "Hello World"
test "POST /echo (chunked)" POST /echo 200 "Hello World" "-H Transfer-Encoding:chunked"
test "POST /invalid" POST /invalid 404
test "PUT /status" PUT /status 405

echo -e "\nPassed: $PASS | Failed: $FAIL"
[ $FAIL -eq 0 ] && exit 0 || exit 1
