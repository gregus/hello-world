def add(a, b):
    return a + b

def subtract(a, b):
    return a - b

def multiply(a, b):
    return a * b

def divide(a, b):
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

def calculator():
    print("Simple Calculator")
    print("-----------------")
    print("Operations: +, -, *, /")
    print("Type 'quit' to exit\n")

    while True:
        expr = input("Enter expression (e.g. 3 + 4): ").strip()
        if expr.lower() == 'quit':
            break

        parts = expr.split()
        if len(parts) != 3:
            print("Invalid input. Use format: number operator number\n")
            continue

        try:
            a, op, b = float(parts[0]), parts[1], float(parts[2])
        except ValueError:
            print("Invalid numbers.\n")
            continue

        try:
            if op == '+':
                result = add(a, b)
            elif op == '-':
                result = subtract(a, b)
            elif op == '*':
                result = multiply(a, b)
            elif op == '/':
                result = divide(a, b)
            else:
                print(f"Unknown operator: {op}\n")
                continue

            print(f"Result: {result}\n")
        except ValueError as e:
            print(f"Error: {e}\n")

if __name__ == "__main__":
    calculator()
