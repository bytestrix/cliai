import subprocess
import time
import os
import sys

def run_test():
    questions_file = "questions.txt"
    results_file = "performance_results.txt"
    binary_path = "./target/debug/cliai"
    
    if not os.path.exists(binary_path):
        print(f"Error: {binary_path} not found. Please build the project first.")
        return

    with open(questions_file, "r") as f:
        questions = [line.strip() for line in f if line.strip()]

    with open(results_file, "w") as f:
        f.write("CLIAI Performance Test Results\n")
        f.write("="*30 + "\n\n")
        f.flush()
        
        total_time = 0
        
        for i, question in enumerate(questions, 1):
            print(f"Testing question {i}/50: {question}")
            sys.stdout.flush()
            
            start_time = time.time()
            try:
                # We use subprocess.run and pass the question as an argument
                # cliai usually outputs the command and explanation
                result = subprocess.run(
                    [binary_path, question],
                    capture_output=True,
                    text=True,
                    timeout=60 # Increased to 60 seconds
                )
                end_time = time.time()
                
                duration = end_time - start_time
                total_time += duration
                
                response = result.stdout.strip()
                error = result.stderr.strip()
                
                f.write(f"Question {i}: {question}\n")
                f.write(f"Time Taken: {duration:.2f} seconds\n")
                f.write(f"Response:\n{response}\n")
                if error:
                    f.write(f"Errors/Warnings:\n{error}\n")
                f.write("-" * 20 + "\n\n")
                f.flush()
                
            except subprocess.TimeoutExpired:
                f.write(f"Question {i}: {question}\n")
                f.write("Status: TIMEOUT\n")
                f.write("-" * 20 + "\n\n")
                f.flush()
                print(f"Timeout on question {i}")
                sys.stdout.flush()
            except Exception as e:
                f.write(f"Question {i}: {question}\n")
                f.write(f"Status: ERROR ({str(e)})\n")
                f.write("-" * 20 + "\n\n")
                f.flush()
                print(f"Error on question {i}: {e}")
                sys.stdout.flush()

        f.write(f"\nTotal Summary\n")
        f.write(f"Total Questions: {len(questions)}\n")
        f.write(f"Total Time: {total_time:.2f} seconds\n")
        f.write(f"Average Time: {total_time/len(questions):.2f} seconds\n")

    print(f"Tests completed. Results saved to {results_file}")

if __name__ == "__main__":
    run_test()
