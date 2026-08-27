import unittest
from space_scorer import SpaceScore, LITERATURE_SCORES

class TestSpaceScorer(unittest.TestCase):
    def test_literature_scores(self):
        self.assertIn("SnSe", LITERATURE_SCORES)
        score = LITERATURE_SCORES["SnSe"]
        self.assertEqual(score.radiation_hardness, 0.80)

    def test_overall_score(self):
        score = SpaceScore(0.5, 0.5, 0.5, 0.5, 0.5, 1.0, "test")
        self.assertAlmostEqual(score.overall_score(), 0.5)

if __name__ == '__main__':
    unittest.main()
