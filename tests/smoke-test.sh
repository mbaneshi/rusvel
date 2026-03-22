#!/bin/bash
# RUSVEL Smoke Test — checks all API endpoints and frontend routes
# Run with: bash tests/smoke-test.sh
# Expects the server to be running on localhost:3000

set -e

BASE="http://localhost:3000"
PASS=0
FAIL=0

check_status() {
    local url="$1"
    local expect="$2"
    local desc="$3"
    local code
    code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null)
    if [ "$code" = "$expect" ]; then
        echo "  [PASS] $desc -> $code"
        PASS=$((PASS+1))
    else
        echo "  [FAIL] $desc -> $code (expected $expect)"
        FAIL=$((FAIL+1))
    fi
}

check_json_field() {
    local url="$1"
    local field="$2"
    local desc="$3"
    local body code
    code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null)
    body=$(curl -s "$url" 2>/dev/null)
    if [ "$code" = "200" ] && echo "$body" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
        echo "  [PASS] $desc -> 200 + valid JSON"
        PASS=$((PASS+1))
    else
        echo "  [FAIL] $desc -> $code"
        FAIL=$((FAIL+1))
    fi
}

echo "============================================"
echo "  RUSVEL Smoke Test"
echo "  $(date '+%Y-%m-%d %H:%M')"
echo "============================================"
echo ""

# Pre-check: is server up?
if ! curl -s -o /dev/null -w "" "$BASE/api/health" 2>/dev/null; then
    echo "  [FATAL] Server not reachable at $BASE"
    exit 1
fi

echo "--- API Endpoints ---"
check_json_field "$BASE/api/health" "status" "Health check"
check_json_field "$BASE/api/sessions" "" "List sessions"

# Create a test session
SESSION_ID=$(curl -s -X POST "$BASE/api/sessions" \
  -H "Content-Type: application/json" \
  -d '{"name":"smoke-test","kind":"General"}' 2>/dev/null \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)

if [ -n "$SESSION_ID" ] && [ "$SESSION_ID" != "" ]; then
    echo "  [PASS] Create session -> $SESSION_ID"
    PASS=$((PASS+1))
else
    echo "  [FAIL] Create session -> no ID returned"
    FAIL=$((FAIL+1))
    SESSION_ID="00000000-0000-0000-0000-000000000000"
fi

check_status "$BASE/api/sessions/$SESSION_ID" "200" "Get session by ID"

# Create a goal
GOAL_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE/api/sessions/$SESSION_ID/mission/goals" \
  -H "Content-Type: application/json" \
  -d '{"title":"Smoke Test Goal","description":"Auto test","timeframe":"Week"}' 2>/dev/null)
if [ "$GOAL_CODE" = "201" ]; then
    echo "  [PASS] Create goal -> 201"
    PASS=$((PASS+1))
else
    echo "  [FAIL] Create goal -> $GOAL_CODE (expected 201)"
    FAIL=$((FAIL+1))
fi

check_status "$BASE/api/sessions/$SESSION_ID/mission/goals" "200" "List goals"
check_status "$BASE/api/sessions/$SESSION_ID/events" "200" "List events"

# Edge cases
check_status "$BASE/api/sessions/not-a-uuid" "400" "Invalid UUID -> 400"
check_status "$BASE/api/sessions/00000000-0000-0000-0000-000000000000" "404" "Missing session -> 404"

echo ""
echo "--- Frontend Routes ---"
check_status "$BASE/" "200" "Dashboard (/)"
check_status "$BASE/forge" "200" "Forge (/forge)"
check_status "$BASE/code" "200" "Code (/code)"
check_status "$BASE/harvest" "200" "Harvest (/harvest)"
check_status "$BASE/content" "200" "Content (/content)"
check_status "$BASE/gtm" "200" "GTM (/gtm)"
check_status "$BASE/settings" "200" "Settings (/settings)"

echo ""
echo "--- Static Assets ---"
FIRST_JS=$(curl -s "$BASE/" 2>/dev/null | grep -o '/_app/[^"]*\.js' | head -1)
if [ -n "$FIRST_JS" ]; then
    check_status "$BASE$FIRST_JS" "200" "JS bundle ($FIRST_JS)"
else
    echo "  [FAIL] No JS bundle found in HTML"
    FAIL=$((FAIL+1))
fi

FIRST_CSS=$(curl -s "$BASE/" 2>/dev/null | grep -o '/_app/[^"]*\.css' | head -1)
if [ -n "$FIRST_CSS" ]; then
    check_status "$BASE$FIRST_CSS" "200" "CSS bundle ($FIRST_CSS)"
else
    echo "  [WARN] No CSS bundle found (may be inline)"
fi

echo ""
echo "============================================"
echo "  Results: $PASS passed, $FAIL failed"
if [ "$FAIL" -eq 0 ]; then
    echo "  ALL GREEN"
else
    echo "  HAS FAILURES"
fi
echo "============================================"

exit $FAIL
