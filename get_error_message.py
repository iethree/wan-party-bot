import random

def get_error_message_for_fun_times_everyone_loves_error_messages():
    with open("data/error_messages.txt") as f:
        lines = f.readlines()
    
    return lines[random.randint(0, len(lines) - 1)]