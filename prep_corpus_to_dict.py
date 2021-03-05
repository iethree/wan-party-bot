
from typing import OrderedDict


sorted_corpus = {}

with open('corpora/cmudict/cmudict', 'r') as corpus:
    # for i in range(10):
    more = True
    i = 0
    while more:
        try:
            line = corpus.readline()
            this_line = line.replace('\n','').split(" ")
            if this_line[0] in sorted_corpus:
                temp = sorted_corpus[this_line[0]]
                temp.append(' '.join(this_line[2:]))
                sorted_corpus[this_line[0]] = temp
            else:
                sorted_corpus[this_line[0]] = [' '.join(this_line[2:])]
            if line == "" or line == "\n":
                more = False
            if i%100 == 0:
                print(f'Loop {i}')
            i += 1
        except:
            break

with open('corpora/cmudict/cmudict.py', 'w') as corpus:
    corpus.write('def return_dict():\n')
    corpus.write('    return ')
    corpus.write(str(sorted_corpus))