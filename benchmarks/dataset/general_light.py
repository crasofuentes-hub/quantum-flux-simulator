def greet(name):
    return "hello " + name

def select_first(items):
    if len(items) == 0:
        return None
    return items[0]