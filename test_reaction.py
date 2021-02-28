import unittest
from reaction import *

class FakeMessage:
    def __init__(self, content):
        self.content = content
        self.reactions = []

    def add_reaction(self, reaction):
        self.reactions.append(reaction)

class TestReaction(unittest.TestCase):
    def test_the_whole_dang_thing(self):
        msg = FakeMessage('poop')
        reaction = Reaction('poop', '💩')
        if reaction.matches(msg.content, msg):
            reaction.apply_to(msg)
        self.assertEqual(['💩'], msg.reactions)

if __name__ == '__main__':
    unittest.main()
