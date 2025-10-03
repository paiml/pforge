# Chapter 19.1: Python Bridge with EXTREME TDD

This chapter demonstrates building a Python-based MCP handler using EXTREME TDD methodology: **5-minute RED-GREEN-REFACTOR cycles** with quality gates.

## Overview

We'll build a text analysis handler in Python that leverages NLP libraries, demonstrating:
- **RED** (2 min): Write failing test
- **GREEN** (2 min): Minimal code to pass
- **REFACTOR** (1 min): Clean up + quality gates
- **COMMIT**: If gates pass

## Prerequisites

```bash
# Install Python bridge dependencies
pip install pforge-python textblob nltk

# Download NLTK data
python -c "import nltk; nltk.download('punkt'); nltk.download('averaged_perceptron_tagger')"
```

## Example: Text Analysis Handler

### Cycle 1: RED - Basic Structure (2 min)

**GOAL**: Create failing test for text word count

```python
# tests/test_text_analyzer.py
import pytest
from handlers import TextAnalyzer

def test_word_count():
    """Test basic word counting."""
    analyzer = TextAnalyzer()
    result = analyzer.handle(text="Hello world")

    assert result["word_count"] == 2
```

**Run test**:
```bash
pytest tests/test_text_analyzer.py::test_word_count
# ❌ FAIL: ModuleNotFoundError: No module named 'handlers'
```

**Time check**: ✅ Under 2 minutes

### Cycle 1: GREEN - Minimal Implementation (2 min)

```python
# handlers.py
from pforge_python import handler

@handler("analyze_text")
class TextAnalyzer:
    def handle(self, text: str) -> dict:
        word_count = len(text.split())
        return {"word_count": word_count}
```

**Run test**:
```bash
pytest tests/test_text_analyzer.py::test_word_count
# ✅ PASS
```

**Time check**: ✅ Under 2 minutes

### Cycle 1: REFACTOR + Quality Gates (1 min)

```bash
# Run quality gates
black handlers.py tests/
mypy handlers.py
pylint handlers.py --max-line-length=100
pytest --cov=handlers --cov-report=term-missing

# Coverage: 100% ✅
# Pylint: 10/10 ✅
# Type check: ✅ Pass
```

**COMMIT**:
```bash
git add handlers.py tests/test_text_analyzer.py
git commit -m "feat: add word count to text analyzer

- Implements basic word counting
- 100% test coverage
- All quality gates pass

🤖 Generated with EXTREME TDD"
```

**Total time**: ✅ 5 minutes

---

## Cycle 2: RED - Sentiment Analysis (2 min)

**GOAL**: Add sentiment analysis

```python
# tests/test_text_analyzer.py
def test_sentiment_analysis():
    """Test sentiment analysis."""
    analyzer = TextAnalyzer()
    result = analyzer.handle(text="I love this amazing product!")

    assert "sentiment" in result
    assert result["sentiment"]["polarity"] > 0  # Positive sentiment
    assert 0 <= result["sentiment"]["subjectivity"] <= 1
```

**Run test**:
```bash
pytest tests/test_text_analyzer.py::test_sentiment_analysis
# ❌ FAIL: KeyError: 'sentiment'
```

**Time check**: ✅ Under 2 minutes

### Cycle 2: GREEN - Add Sentiment (2 min)

```python
# handlers.py
from pforge_python import handler
from textblob import TextBlob

@handler("analyze_text")
class TextAnalyzer:
    def handle(self, text: str) -> dict:
        word_count = len(text.split())

        # Add sentiment analysis
        blob = TextBlob(text)

        return {
            "word_count": word_count,
            "sentiment": {
                "polarity": blob.sentiment.polarity,
                "subjectivity": blob.sentiment.subjectivity,
            },
        }
```

**Run test**:
```bash
pytest tests/test_text_analyzer.py::test_sentiment_analysis
# ✅ PASS
```

**Time check**: ✅ Under 2 minutes

### Cycle 2: REFACTOR + Quality Gates (1 min)

```bash
# Quality gates
black handlers.py tests/
pytest --cov=handlers --cov-report=term-missing

# Coverage: 100% ✅
# All tests: 2/2 passing ✅
```

**COMMIT**:
```bash
git add handlers.py tests/test_text_analyzer.py
git commit -m "feat: add sentiment analysis

- TextBlob integration for polarity/subjectivity
- 100% test coverage maintained
- All tests passing (2/2)

🤖 Generated with EXTREME TDD"
```

**Total time**: ✅ 5 minutes

---

## Cycle 3: RED - Noun Phrase Extraction (2 min)

```python
# tests/test_text_analyzer.py
def test_noun_phrases():
    """Test noun phrase extraction."""
    analyzer = TextAnalyzer()
    result = analyzer.handle(text="The quick brown fox jumps over the lazy dog")

    assert "noun_phrases" in result
    assert isinstance(result["noun_phrases"], list)
    assert len(result["noun_phrases"]) > 0
```

**Run test**:
```bash
pytest tests/test_text_analyzer.py::test_noun_phrases
# ❌ FAIL: KeyError: 'noun_phrases'
```

**Time check**: ✅ Under 2 minutes

### Cycle 3: GREEN - Extract Noun Phrases (2 min)

```python
# handlers.py
from pforge_python import handler
from textblob import TextBlob

@handler("analyze_text")
class TextAnalyzer:
    def handle(self, text: str) -> dict:
        word_count = len(text.split())
        blob = TextBlob(text)

        return {
            "word_count": word_count,
            "sentiment": {
                "polarity": blob.sentiment.polarity,
                "subjectivity": blob.sentiment.subjectivity,
            },
            "noun_phrases": list(blob.noun_phrases),
        }
```

**Run test**:
```bash
pytest tests/test_text_analyzer.py::test_noun_phrases
# ✅ PASS (3/3)
```

**Time check**: ✅ Under 2 minutes

### Cycle 3: REFACTOR + Quality Gates (1 min)

**Refactor**: Extract blob creation to avoid repetition

```python
# handlers.py
from pforge_python import handler
from textblob import TextBlob

@handler("analyze_text")
class TextAnalyzer:
    def handle(self, text: str) -> dict:
        blob = self._create_blob(text)

        return {
            "word_count": len(text.split()),
            "sentiment": {
                "polarity": blob.sentiment.polarity,
                "subjectivity": blob.sentiment.subjectivity,
            },
            "noun_phrases": list(blob.noun_phrases),
        }

    def _create_blob(self, text: str) -> TextBlob:
        """Create TextBlob instance for analysis."""
        return TextBlob(text)
```

**Quality gates**:
```bash
black handlers.py
pylint handlers.py --max-line-length=100
pytest --cov=handlers --cov-report=term-missing

# Coverage: 100% ✅
# Pylint: 10/10 ✅
# All tests: 3/3 ✅
```

**COMMIT**:
```bash
git add handlers.py tests/test_text_analyzer.py
git commit -m "feat: add noun phrase extraction

- Extract noun phrases using TextBlob
- Refactor: extract blob creation helper
- Maintain 100% coverage (3/3 tests)

🤖 Generated with EXTREME TDD"
```

**Total time**: ✅ 5 minutes

---

## Integration with pforge

### Configuration (forge.yaml)

```yaml
forge:
  name: python-nlp-server
  version: 0.1.0
  transport: stdio

tools:
  - type: native
    name: analyze_text
    description: "Analyze text with NLP: word count, sentiment, noun phrases"
    handler:
      path: python:handlers.TextAnalyzer
    params:
      text:
        type: string
        required: true
        description: "Text to analyze"
```

### Running the Server

```bash
# Build server
pforge build --release

# Run server
pforge serve

# Test via MCP client
echo '{"text": "I love this amazing product!"}' | pforge test analyze_text
```

**Output**:
```json
{
  "word_count": 5,
  "sentiment": {
    "polarity": 0.65,
    "subjectivity": 0.85
  },
  "noun_phrases": [
    "amazing product"
  ]
}
```

---

## Quality Metrics

### Final Coverage Report

```
Name              Stmts   Miss  Cover   Missing
-----------------------------------------------
handlers.py          12      0   100%
tests/__init__.py     0      0   100%
tests/test_text_analyzer.py  15      0   100%
-----------------------------------------------
TOTAL                27      0   100%
```

### Complexity Analysis

```bash
radon cc handlers.py -a
# handlers.py
#   C 1:0 TextAnalyzer._create_blob - A (1)
#   C 1:0 TextAnalyzer.handle - A (2)
# Average complexity: A (1.5) ✅
```

### Type Coverage

```bash
mypy handlers.py --strict
# Success: no issues found in 1 source file ✅
```

---

## Development Workflow Summary

**Total development time**: 15 minutes (3 cycles × 5 min)

**Commits**: 3 clean commits, all tests passing

**Quality maintained**:
- ✅ 100% test coverage throughout
- ✅ All quality gates passing
- ✅ Complexity: A grade
- ✅ Type safety: 100%

**Key Principles Applied**:
1. **Jidoka** ("stop the line"): Quality gates prevent bad commits
2. **Kaizen** (continuous improvement): Each cycle adds value
3. **Respect for People**: Clear, readable code
4. **Built-in Quality**: TDD ensures correctness

---

## Troubleshooting

### Common Issues

**Import errors**:
```bash
# Ensure pforge-python is in PYTHONPATH
export PYTHONPATH=$PWD/bridges/python:$PYTHONPATH
```

**NLTK data missing**:
```bash
python -c "import nltk; nltk.download('all')"
```

**Coverage not at 100%**:
```bash
# Check what's missing
pytest --cov=handlers --cov-report=html
open htmlcov/index.html
```

---

## Summary

This chapter demonstrated:
- ✅ EXTREME TDD with 5-minute cycles
- ✅ Python bridge integration
- ✅ NLP library usage (TextBlob)
- ✅ 100% test coverage maintained
- ✅ Quality gates enforced
- ✅ Clean commit history

**Next**: Chapter 19.2 - Go Bridge with EXTREME TDD
