def listify(maybe_list):
    return maybe_list if isinstance(maybe_list, type([])) else [maybe_list]

class Reaction:
    def __init__(self, keywords, reactions):
        self.keywords = listify(keywords)
        self.reactions = listify(reactions)

    def matches(self, content, message=None):
        return any(kw in content for kw in self.keywords)

    async def apply_to(self, message):
        for reaction in self.reactions:
            print("Adding {}!".format(reaction))
            await message.add_reaction(reaction)

class MatchingReaction(Reaction):
    def __init__(self, matcher, reactions):
        self.matcher = matcher
        super().__init__(None, reactions)

    def matches(self, content, message=None):
        return self.matcher(content, message)

