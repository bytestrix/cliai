#!/bin/bash

# CLIAI Comprehensive Test Suite
# Tests Local models with improved questions and context support
# Single test file with single results file for easy comparison

# Configuration
LOCAL_MODEL="qwen2.5-coder:0.5b"
OUTPUT_FILE="cliai_test_results.md"
CLIAI_BIN="./target/release/cliai"
TOTAL_START=$(date +%s%N)

# Test counters
declare -A RESULTS
RESULTS[local_passed]=0
RESULTS[local_failed]=0
RESULTS[local_partial]=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Function to check if a line looks like a command
is_likely_command() {
    local line="$1"
    local first_word=$(echo "$line" | awk '{print $1}')
    case "$first_word" in
        ls|find|mkdir|cat|grep|cp|mv|rm|chmod|ps|kill|git|ping|curl|wget|df|du|free|top|htop|whoami|hostname|uptime|uname|which|echo|touch|head|tail|wc|sort|uniq|awk|sed|test|stat|lsblk|fdisk|netstat|ss|lsof|journalctl|systemctl|pacman|apt|yay|paru|pip|npm|cargo|make|gcc|clang|python|node|java)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Function to extract command from AI response
extract_command() {
    local result="$1"
    local cmd=""
    
    # Method 1: Look for "Command:" in the result
    if echo "$result" | grep -q "Command:"; then
        cmd=$(echo "$result" | grep "Command:" | head -1 | sed 's/.*Command: //' | sed 's/^`//;s/`$//')
    # Method 2: Look for "🚀 Executing:" pattern
    elif echo "$result" | grep -q "🚀 Executing:"; then
        cmd=$(echo "$result" | grep "🚀 Executing:" | head -1 | sed 's/.*🚀 Executing: //' | sed 's/^`//;s/`$//')
    # Method 3: Look for backtick-wrapped commands
    elif [[ "$result" =~ \`([^\`]+)\` ]]; then
        cmd="${BASH_REMATCH[1]}"
    # Method 4: Look for the first line that looks like a command
    else
        local first_line=$(echo "$result" | head -1 | grep -v "⚠️" | grep -v "💡" | grep -v "🚀" | grep -v "total" | grep -v "drwx" | grep -v "^$")
        if [ -n "$first_line" ] && is_likely_command "$first_line"; then
            cmd="$first_line"
        fi
    fi
    
    echo "$cmd"
}

# Function to test question
test_question() {
    local num=$1
    local category=$2
    local question=$3
    local expected_pattern=$4
    local should_execute=${5:-false}
    
    echo -e "${CYAN}=== Question ${num}: ${question} ===${NC}"
    
    # Write question header to results
    echo "## Question ${num}" >> "$OUTPUT_FILE"
    echo "**Category:** ${category}" >> "$OUTPUT_FILE"
    echo "**Question:** ${question}" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Test with Local model
    echo -e "${YELLOW}Testing with Local Model...${NC}"
    test_with_local "$num" "$category" "$question" "$expected_pattern" "$should_execute"
    
    echo "---" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo ""
}

# Function to test with Local model
test_with_local() {
    local num=$1
    local category=$2
    local question=$3
    local expected_pattern=$4
    local should_execute=$5
    
    # Switch to local model
    $CLIAI_BIN select "$LOCAL_MODEL" > /dev/null 2>&1
    
    local start=$(date +%s%N)
    local result
    
    if timeout 30s $CLIAI_BIN "$question" > /tmp/local_${num} 2>&1; then
        result=$(cat /tmp/local_${num})
    else
        result="⏱️ Timeout Error: Local model timed out"
    fi
    
    local end=$(date +%s%N)
    local duration=$(( (end - start) / 1000000 ))
    
    # Extract command
    local cmd=$(extract_command "$result")
    
    # Determine status
    local status="✓ Success"
    local execution_result=""
    
    if [[ "$result" == *"Timeout"* ]]; then
        status="✗ Failed - Timeout"
        ((RESULTS[local_failed]++))
    elif [ -z "$cmd" ] || [ "$cmd" = "(none)" ]; then
        if [[ "$category" == "SHELL" ]]; then
            status="✗ Failed - No command"
            ((RESULTS[local_failed]++))
        else
            status="✓ Success - Explanation"
            ((RESULTS[local_passed]++))
        fi
    else
        # Check expected pattern if provided
        if [ -n "$expected_pattern" ]; then
            if echo "$cmd" | grep -qE "$expected_pattern"; then
                status="✓ Success"
                ((RESULTS[local_passed]++))
            else
                status="⚠ Partial - Pattern mismatch"
                ((RESULTS[local_partial]++))
            fi
        else
            status="✓ Success"
            ((RESULTS[local_passed]++))
        fi
        
        # Execute command if requested and safe
        if [ "$should_execute" = "true" ] && is_safe_to_execute "$cmd"; then
            echo "  → Executing: $cmd"
            if timeout 10s bash -c "$cmd" > /tmp/exec_local_${num} 2>&1; then
                execution_result="✓ Executed successfully"
            else
                execution_result="✗ Execution failed"
            fi
            rm -f /tmp/exec_local_${num}
        fi
    fi
    
    echo "  Local: '$cmd' - $status (${duration}ms)"
    if [ -n "$execution_result" ]; then
        echo "  $execution_result"
    fi
    
    # Write to results
    echo "### Local Model Results" >> "$OUTPUT_FILE"
    echo "**Model:** $LOCAL_MODEL" >> "$OUTPUT_FILE"
    echo "**Command:** \`${cmd}\`" >> "$OUTPUT_FILE"
    echo "**Status:** ${status}" >> "$OUTPUT_FILE"
    echo "**Response Time:** ${duration}ms" >> "$OUTPUT_FILE"
    if [ -n "$execution_result" ]; then
        echo "**Execution:** ${execution_result}" >> "$OUTPUT_FILE"
    fi
    echo "**Full Response:** \`${result:0:200}...\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    rm -f /tmp/local_${num}
}

# Function to check if command is safe to execute
is_safe_to_execute() {
    local cmd="$1"
    local dangerous_patterns=(
        "rm -rf" "sudo" "chmod 777" "dd if=" "mkfs" "fdisk" 
        "reboot" "shutdown" "kill -9" ">/dev/" "format" "shred"
        "curl.*|.*sh" "wget.*|.*bash"
    )
    
    for pattern in "${dangerous_patterns[@]}"; do
        if [[ "$cmd" == *"$pattern"* ]]; then
            return 1
        fi
    done
    return 0
}

# Function to test context support
test_context_support() {
    echo -e "${CYAN}=== Testing Context Support ===${NC}"
    echo "## Context Support Tests" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Test 1: Ask a command, then ask about it
    echo -e "${BLUE}Context Test 1: Command follow-up${NC}"
    
    # Clear history first
    $CLIAI_BIN clear > /dev/null 2>&1
    
    # First question
    echo "  → First: 'list all files'"
    result1=$($CLIAI_BIN "list all files" 2>&1)
    cmd1=$(extract_command "$result1")
    echo "  → Got: '$cmd1'"
    
    # Follow-up question
    echo "  → Follow-up: 'what does the previous command do?'"
    result2=$($CLIAI_BIN "what does the previous command do?" 2>&1)
    echo "  → Response: '${result2:0:100}...'"
    
    # Write context test results
    echo "### Context Test 1: Command Follow-up" >> "$OUTPUT_FILE"
    echo "**First Question:** list all files" >> "$OUTPUT_FILE"
    echo "**First Response:** \`$cmd1\`" >> "$OUTPUT_FILE"
    echo "**Follow-up Question:** what does the previous command do?" >> "$OUTPUT_FILE"
    echo "**Follow-up Response:** \`${result2:0:200}...\`" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    echo "---" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
}

echo -e "${GREEN}🚀 CLIAI Comprehensive Test Suite${NC}"
echo "Testing Local models with improved questions"
echo ""

# Initialize results file
echo "# CLIAI Comprehensive Test Results" > "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "**Test Date:** $(date)" >> "$OUTPUT_FILE"
echo "**Local Model:** $LOCAL_MODEL" >> "$OUTPUT_FILE"
echo "**Binary:** $CLIAI_BIN" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "---" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Clear history
$CLIAI_BIN clear > /dev/null 2>&1

# Create test environment
mkdir -p test_env
echo "test content" > test_env/sample.txt
echo "print('Hello World')" > test_env/hello.py
echo "fn main() { println!(\"Hello\"); }" > test_env/main.rs

# Run improved test questions
echo "=== File Operations ===" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

test_question 1 "SHELL" "list all files in current directory including hidden ones" "ls.*-.*a" true
test_question 2 "SHELL" "create a new directory called 'projects'" "mkdir.*projects" true
test_question 3 "SHELL" "find all .rs files in current directory" "find.*\.rs" true

echo "=== System Information ===" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

test_question 6 "SHELL" "show current system hostname" "hostname" true
test_question 7 "SHELL" "display kernel version" "uname.*-r" true

# Test context support
test_context_support

# Calculate total time
TOTAL_END=$(date +%s%N)
TOTAL_DURATION=$(( (TOTAL_END - TOTAL_START) / 1000000 ))

# Write final summary
echo "## Final Summary" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "### Local Model Performance" >> "$OUTPUT_FILE"
echo "- **Passed:** ${RESULTS[local_passed]}/5" >> "$OUTPUT_FILE"
echo "- **Partial:** ${RESULTS[local_partial]}/5" >> "$OUTPUT_FILE"
echo "- **Failed:** ${RESULTS[local_failed]}/5" >> "$OUTPUT_FILE"
echo "- **Success Rate:** $(( (RESULTS[local_passed] + RESULTS[local_partial]) * 100 / 5 ))%" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Cleanup
rm -rf test_env

echo ""
echo -e "${GREEN}✅ Comprehensive test completed!${NC}"
echo -e "${BLUE}Local Results:${NC} ${RESULTS[local_passed]} passed, ${RESULTS[local_partial]} partial, ${RESULTS[local_failed]} failed"
echo -e "${BLUE}Local Success Rate:${NC} $(( (RESULTS[local_passed] + RESULTS[local_partial]) * 100 / 5 ))%"
echo -e "${BLUE}Results saved to:${NC} $OUTPUT_FILE"
echo -e "${BLUE}Total time:${NC} ${TOTAL_DURATION}ms ($((TOTAL_DURATION / 1000))s)"