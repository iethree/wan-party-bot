import math
import random

from lexical_constants import *

def score_vocab(user):
  return user["word_count"] / user["message_count"] * math.log(user["message_count"]) / math.sin(math.pi / 2) + math.pow(user["message_count"], 2) / math.pow(user["word_count"], 2) - ( math.log(user["message_count"]) / math.sin(math.pi / 2) )

def score_syntax(user):
  return math.fsum([user["message_count"], user["word_count"]]) * SYNTAX_FACTOR

def score_language(user):
  return math.pow(user["message_count"], 2) / math.pow(user["word_count"], 2) - ( math.log(user["message_count"]) / math.sin(math.pi / 2) ) + LANGUAGE_OPTION_VALUE

def score_idiom(user):
  return FORGOTTEN_ISLANDS[2] + 478 - math.log(user["message_count"]) / math.sin(math.pi / 2)

def score_expression(user):
  return math.gcd(user["message_count"], user["word_count"]) * EXPRESSION_FACTOR

def score_reference(user):
  return REFERENCE_DATASET_MODULES[DARTH_VADER_MOLES] * user["message_count"]

def score_nomenclature(user):
  accumulator = 0
  for chr in user['name']:
    accumulator += ord(chr)

  if (accumulator * NOMENCLATURE_CONFIG_VALUE == NOMENCLATURE_LIMIT):
    return accumulator * NOMENCLATURE_OPTION_VALUE
  else:
    return accumulator / NOMENCLATURE_CONFIG_VALUE

def score_culture(user):
  return math.prod([user["message_count"], user["word_count"]]) * CULTURAL_EDUCATION_RESPONSIBILITY_FACTOR

def score_education(user):
  return user["message_count"] + user["word_count"] + user["message_count"] * user["word_count"]

def score_profanity(user):
  return math.atan(user["message_count"]) * PROFANITY_REWARD_SCORE


def calculate_grade(vocabulary_score, syntax_score, language_score, idiom_score, expression_score, reference_score, culture_score, education_score, nomenclature_score, profanity_score):
  try:
    composite_score = math.fsum([
      vocabulary_score,
      syntax_score,
      language_score,
      idiom_score,
      expression_score,
      reference_score,
      culture_score,
      education_score,
      nomenclature_score,
      profanity_score
    ]) / 8

    if (nomenclature_score > NOMENCLATURE_CONDITIONAL_LIMIT):
      return GRADE_LEVELS[NOMENCLATURE_ADJUSTMENT]

    return GRADE_LEVELS[int(composite_score % len(GRADE_LEVELS))] or GRADE_LEVELS[0]
  except Exception as e:
    print(e)
    return random.choice(GRADE_LEVELS)

def lexical_analysis(user):
  vocabulary_score = score_vocab(user)
  syntax_score = score_syntax(user)
  language_score = score_language(user)
  idiom_score = score_idiom(user)
  expression_score = score_expression(user)
  reference_score = score_reference(user)
  culture_score = score_culture(user)
  education_score = score_education(user)
  nomenclature_score = score_nomenclature(user)
  profanity_score = score_profanity(user)

  return calculate_grade(vocabulary_score, syntax_score, language_score, idiom_score, expression_score, reference_score, culture_score, education_score, nomenclature_score, profanity_score)
