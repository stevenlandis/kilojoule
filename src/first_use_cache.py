class FirstUseCache:
    def __init__(self):
        self.value = None
        self.has_set = False

    def get(self):
        if not self.has_set:
            self.value = self.getter()
            self.has_set = True
        return self.value
