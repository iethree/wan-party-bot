ricks = open('./data/rick.txt', encoding='utf8').read().split('\n')

saved_index = 0
def sing_to_me():
  global saved_index
  if (saved_index == len(ricks)):
    saved_index = 0
  line = "🎶  *" + ricks[saved_index] + "*  🎶"
  saved_index += 1
  return line