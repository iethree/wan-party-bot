import sys
import logging
import random
from collections import defaultdict
from count_syllables_discord import count_syllables
from corpora.cmudict import cmudict

logging.disable(logging.CRITICAL)  # Comment out to enable debugging messages
logging.basicConfig(level=logging.DEBUG, format="%(message)s")


# def load_training_file(file: str):
#     """Return text file as a string."""
#     with open(file) as f:
#         raw_haiku = f.read()
#         return raw_haiku


def prep_training(raw_haiku: str):
    """Load string, remove newline, split words on spaces, and return list."""
    corpus = raw_haiku.replace("\n", " ").split()
    # remove words not in dictionary
    cmudict = cmudict.return_dict()
    filtered_corpus = [word for word in corpus if word in cmudict.keys()]
    return filtered_corpus


def map_word_to_word(corpus: list):
    """Load list and use dictionary to map word to word that follows."""
    limit = len(corpus) - 1
    dict1_to_1 = defaultdict(list)
    for index, word in enumerate(corpus):
        if index < limit:
            suffix = corpus[index + 1]
            dict1_to_1[word].append(suffix)
    logging.debug(
        f"map word to word results for \"america\" = {dict1_to_1['america']}\n"
    )
    return dict1_to_1


def map_2_words_to_word(corpus: list):
    """Load list and use dictionary to map word-pair to trailing word."""
    limit = len(corpus) - 2
    dict2_to_1 = defaultdict(list)
    for index, word in enumerate(corpus):
        if index < limit:
            key = word + " " + corpus[index + 1]
            suffix = corpus[index + 2]
            dict2_to_1[key].append(suffix)
    logging.debug(
        f"map word to word results for \"american soldier\" = {dict2_to_1['american soldier']}\n"
    )
    return dict2_to_1


def random_word(corpus: list):
    """Return random word and syllable count from training corpus."""
    word = random.choice(corpus)
    num_syls = count_syllables(word)
    if num_syls > 4:
        return random_word(corpus)
    else:
        logging.debug(f"random word and syllables = {word} {num_syls}\n")
        return (word, num_syls)


def word_after_single(
    prefix: str, suffix_map_1: dict, current_syls: int, target_syls: int
) -> list:
    """Returns all acceptable words in a corpus that follow a single word."""
    accepted_words = []
    suffixes = suffix_map_1.get(prefix)
    if suffixes != None:
        for candidate in suffixes:
            num_syls = count_syllables(candidate)
            if current_syls + num_syls <= target_syls:
                accepted_words.append(candidate)
    logging.debug(f'accepted words after "{prefix}" = {set(accepted_words)}\n')
    return accepted_words


def word_after_double(
    prefix: str, suffix_map_2: dict, current_syls: int, target_syls: int
) -> list:
    """Returns all acceptable words in a corpus that follow a word pair."""
    accepted_words = []
    suffixes = suffix_map_2.get(prefix)
    if suffixes != None:
        for candidate in suffixes:
            num_syls = count_syllables(candidate)
            if current_syls + num_syls <= target_syls:
                accepted_words.append(candidate)
    logging.debug(f'accepted words after "{prefix}" = {set(accepted_words)}\n')
    return accepted_words


def haiku_line(
    suffix_map_1: dict,
    suffix_map_2: dict,
    corpus: list,
    end_prev_line,
    target_syls: int,
):
    """Build a haiku line from a training corpus and return it."""
    line = "2/3"
    line_syls = 0
    current_line = []

    if len(end_prev_line) == 0:  # build first line
        line = "1"
        word, num_syls = random_word(corpus)
        current_line.append(word)
        line_syls += num_syls
        word_choices = word_after_single(word, suffix_map_1, line_syls, target_syls)
        while len(word_choices) == 0:
            prefix = random.choice(corpus)
            logging.debug(f"new random prefix = {prefix}\n")
            word_choices = word_after_single(
                prefix, suffix_map_1, line_syls, target_syls
            )
        word = random.choice(word_choices)
        num_syls = count_syllables(word)
        logging.debug(f"word and syllables {word}: {num_syls}\n")
        line_syls += num_syls
        current_line.append(word)
        if line_syls == target_syls:
            end_prev_line.extend(current_line[-2:])
            return current_line, end_prev_line

    else:  # build lines 2 and 3
        current_line.extend(end_prev_line)

    while True:
        logging.debug(f"line = {line}\n")
        prefix = current_line[-2] + " " + current_line[-1]
        word_choices = word_after_double(prefix, suffix_map_2, line_syls, target_syls)

        while len(word_choices) == 0:
            index = random.randint(0, len(corpus) - 2)
            prefix = corpus[index] + " " + corpus[index + 1]
            logging.debug(f"new random prefix = {prefix}")
            word_choices = word_after_double(
                prefix, suffix_map_2, line_syls, target_syls
            )

        word = random.choice(word_choices)
        num_syls = count_syllables(word)
        logging.debug(f"word and syllables = {word}: {num_syls}")

        if line_syls + num_syls > target_syls:
            continue
        elif line_syls + num_syls < target_syls:
            current_line.append(word)
            line_syls += num_syls
        elif line_syls + num_syls == target_syls:
            current_line.append(word)
            break

    end_prev_line = []
    end_prev_line.extend(current_line[-2:])

    if line == "1":
        final_line = current_line[:]
    else:
        final_line = current_line[2:]

    return final_line, end_prev_line


def main():
    """Give user choice of building a haiku or modifying an existing haiku."""
    intro = """\n
    The best haikus.\n"""
    print(intro)

    raw_haiku = load_training_file("train_trump.txt")
    corpus = prep_training(raw_haiku)
    suffix_map_1 = map_word_to_word(corpus)
    suffix_map_2 = map_2_words_to_word(corpus)
    final = []

    choice = None
    while choice != "0":

        print(
            """
        Japanese Haiku Generator
        
        0 - Quit
        1 - Generate a Haiku
        2 - Regenerate Line 2
        3 - Regenerate Line 3
        """
        )
        choice = input("Choice: ")
        print()

        # exit
        if choice == "0":
            print("Sayanora.")
            sys.exit()

        # generate a full haiku
        elif choice == "1":
            final = []
            end_prev_line = []
            first_line, end_prev_line1 = haiku_line(
                suffix_map_1, suffix_map_2, corpus, end_prev_line, 5
            )
            final.append(first_line)
            line, end_prev_line2 = haiku_line(
                suffix_map_1, suffix_map_2, corpus, end_prev_line1, 7
            )
            final.append(line)
            line, end_prev_line3 = haiku_line(
                suffix_map_1, suffix_map_2, corpus, end_prev_line2, 5
            )
            final.append(line)

        # regenerate line 2
        elif choice == "2":
            if not final:
                print("Please generate a full haiku first (Option 1).")
                continue
            else:
                line, end_prev_line2 = haiku_line(
                    suffix_map_1, suffix_map_2, corpus, end_prev_line1, 7
                )
                final[1] = line

        # regenerate line 3
        elif choice == "3":
            if not final:
                print("Please generate a full haiku first (Option 1).")
                continue
            else:
                line, end_prev_line3 = haiku_line(
                    suffix_map_1, suffix_map_2, corpus, end_prev_line2, 5
                )
                final[2] = line
        # some unknown choice
        else:
            print("Sorry, but that isn't a valid choice.", sys.stderr)
            continue

        # display results
        print()
        print("First line = ", end="")
        print(" ".join(final[0]))
        print("Second line = ", end="")
        print(" ".join(final[1]))
        print("Third line = ", end="")
        print(" ".join(final[2]))
        print()

    input("\n\nPress Enter key to exit.")


if __name__ == "__main__":
    main()

def gen_haiku(training_file):
    
    raw_haiku = training_file
    corpus = prep_training(raw_haiku)
    suffix_map_1 = map_word_to_word(corpus)
    suffix_map_2 = map_2_words_to_word(corpus)
    final = []

    # generate a full haiku
    final = []
    end_prev_line = []
    first_line, end_prev_line1 = haiku_line(
        suffix_map_1, suffix_map_2, corpus, end_prev_line, 5
    )
    final.append(first_line)
    line, end_prev_line2 = haiku_line(
        suffix_map_1, suffix_map_2, corpus, end_prev_line1, 7
    )
    final.append(line)
    line, end_prev_line3 = haiku_line(
        suffix_map_1, suffix_map_2, corpus, end_prev_line2, 5
    )
    final.append(line)
    
    return final