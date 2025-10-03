#!/usr/bin/env python3
"""
Simple sentiment analyzer for polyglot example
In production, this would use a real NLP library like TextBlob or transformers
"""

import sys
import json


def analyze_sentiment(text, language="en"):
    """
    Simple rule-based sentiment analysis
    In production, use proper NLP libraries
    """
    text_lower = text.lower()

    # Simple positive/negative word lists
    positive_words = ['good', 'great', 'excellent', 'wonderful', 'amazing', 'fantastic', 'love', 'best']
    negative_words = ['bad', 'terrible', 'awful', 'horrible', 'worst', 'hate', 'poor']

    positive_count = sum(1 for word in positive_words if word in text_lower)
    negative_count = sum(1 for word in negative_words if word in text_lower)

    if positive_count > negative_count:
        sentiment = "positive"
        score = min(0.5 + (positive_count * 0.15), 1.0)
    elif negative_count > positive_count:
        sentiment = "negative"
        score = max(-0.5 - (negative_count * 0.15), -1.0)
    else:
        sentiment = "neutral"
        score = 0.0

    return {
        "sentiment": sentiment,
        "score": score,
        "language": language,
        "positive_words_found": positive_count,
        "negative_words_found": negative_count
    }


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Text argument required"}))
        sys.exit(1)

    text = sys.argv[1]
    language = sys.argv[2] if len(sys.argv) > 2 else "en"

    result = analyze_sentiment(text, language)
    print(json.dumps(result))
