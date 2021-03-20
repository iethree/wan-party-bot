"""
Contains rhyme() function which generates a list of rhymes for a given word.


Phoneme Example Translation    Phoneme Example Translation
    ------- ------- -----------    ------- ------- -----------
    AA      odd     AA D           AE      at      AE T
    AH      hut     HH AH T        AO      ought   AO T
    AW      cow     K AW           AY      hide    HH AY D
    B       be      B IY           CH      cheese  CH IY Z
    D       dee     D IY           DH      thee    DH IY
    EH      Ed      EH D           ER      hurt    HH ER T
    EY      ate     EY T           F       fee     F IY
    G       green   G R IY N       HH      he      HH IY
    IH      it      IH T           IY      eat     IY T
    JH      gee     JH IY          K       key     K IY
    L       lee     L IY           M       me      M IY
    N       knee    N IY           NG      ping    P IH NG
    OW      oat     OW T           OY      toy     T OY
    P       pee     P IY           R       read    R IY D
    S       sea     S IY           SH      she     SH IY
    T       tea     T IY           TH      theta   TH EY T AH
    UH      hood    HH UH D        UW      two     T UW
    V       vee     V IY           W       we      W IY
    Y       yield   Y IY L D       Z       zee     Z IY
    ZH      seizure S IY ZH ER
"""

if __name__ == "__main__":
    import cmudict
else:
    import corpora.cmudict.cmudict as cmudict

PHONEMES = {
    "AA",
    "AE",
    "AH",
    "AO",
    "AW",
    "AY",
    "B",
    "CH",
    "D",
    "DH",
    "EH",
    "ER",
    "EY",
    "F",
    "G",
    "HH",
    "IH",
    "IY",
    "JH",
    "K",
    "L",
    "M",
    "N",
    "NG",
    "OW",
    "OY",
    "P",
    "R",
    "S",
    "SH",
    "T",
    "TH",
    "UH",
    "UW",
    "UW",
    "V",
    "W",
    "Y",
    "Z",
    "ZH",
}

VOWEL_PHONEMES = {
    "AA",
    "AE",
    "AH",
    "AO",
    "AW",
    "AY",
    "EH",
    "ER",
    "EY",
    "IH",
    "IY",
    "OW",
    "OY",
    "UH",
    "UW",
    "UW",
}

CONSONANT_PHONEMES = PHONEMES.difference(VOWEL_PHONEMES)

DIGITS = [str(x) for x in range(0, 10)]

# complete this to generate a dictionary containing rhyming words


def get_phoneme_count_to_rhyme(phoneme_input: str) -> int:
    """
    1: End Rhymes (blue/shoe) final vowel and following consonant sounds
    2: Last Syllable Rhymes (timber/harbor) consonant, vowel, consonant
    3: Double Rhyme (conviction/prediction) second to last syllable and all following
    4: Triple Rhyme (transportation/dissertation) triple to last syllable and all following
    """
    # get a count of syllables in word
    syllable_count = 0
    syllable_location = []
    vowel_location = []
    phoneme_filtered = []
    for i, phoneme in enumerate(phoneme_input):
        for digit in DIGITS:
            if digit in phoneme:
                syllable_count += 1
                phoneme_filtered.append(phoneme.replace(digit, ""))
                syllable_location.append(i)
                break
            if digit == "9":
                phoneme_filtered.append(phoneme)
                vowel_location.append(i)

    count = 0
    results = []

    try:
        # first case in description
        results.append(len(phoneme_input) - vowel_location[-1])
        # second case in description
        results.append(len(phoneme_input) - syllable_location[-1])
        # third case in description
        results.append(len(phoneme_input) - syllable_location[-2])
        # fourth case in description
        results.append(len(phoneme_input) - syllable_location[-3])
    except IndexError:
        pass

    return results


def rhyme(word: str, rhyming_type: int = 0) -> dict:
    """
    Returns a dictionary containing rhymes to the word. If there are multiple valid pronunciations,
    it will return with 2 keys, one for each pronunciation.
    >>> rhyme(pool)
    >>> {pool: [rhyme1, rhyme2, etc...]}
    """
    DICTIONARY = cmudict.return_dict()

    rhyming_dictionary = {word.upper(): []}

    # GET FINAL SOUNDS IN WORD TO RHYME
    try:
        pronunciation = DICTIONARY[word.upper()]
        for a_pronunciation in pronunciation:
            word_phonemes = a_pronunciation.split(" ")
            phoneme_target = get_phoneme_count_to_rhyme(word_phonemes)
            word_phonemes_count = len(word_phonemes)
            while True:
                matches_count = 0
                if rhyming_type == 0:
                    rhyming_target = phoneme_target[-1]
                else:
                    if len(phoneme_target) >= rhyming_type - 1:
                        rhyming_target = phoneme_target[rhyming_type - 1]
                    else:
                        rhyming_target = phoneme_target[-1]
                target_sounds = " ".join(
                    word_phonemes[word_phonemes_count - rhyming_target :]
                )
                target_sounds = "".join([i for i in target_sounds if i not in DIGITS])
                for potential_match in DICTIONARY.keys():
                    for match_variation in DICTIONARY[potential_match]:
                        if "".join(
                            [i for i in match_variation if i not in DIGITS]
                        ).endswith(target_sounds):
                            # skip matching the same word or alternate spellings
                            # e.g. tomato shouldn't match tomatoe
                            if potential_match == word.upper() or potential_match == (
                                word.upper() + "E"
                            ):
                                continue
                            rhyming_dictionary[word.upper()].append(potential_match)
                            matches_count += 1
                if rhyming_type == 1:
                    break
                elif matches_count < 1:
                    rhyming_type -= 1
                else:
                    break

        return rhyming_dictionary
    except IndexError:
        return []


def test_rhyme() -> str:
    result = rhyme("sibling")
    for pronunciation in result.keys():
        print(
            pronunciation,
            " rhymes with:\n",
            result[pronunciation],
            f"\n\nA total of {len(result[pronunciation])}",
        )


if __name__ == "__main__":
    test_rhyme()