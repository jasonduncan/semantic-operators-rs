#!/usr/bin/env python3
"""Export synthetic recipe/frame/region expectations from the exact pinned Python source.

No SDK, credentials, network, or model calls are used. The reference checkout is
read-only; bytecode writes are disabled. Image UUIDs are normalized in outputs.
"""
import argparse
import base64
import dataclasses
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source = args.source.resolve()
    baseline = json.loads((ROOT / "docs/source-baseline.json").read_text())["sources"][0]
    revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    if revision != baseline["revision"]:
        parser.error("reference revision differs from pinned baseline")
    if subprocess.check_output(["git", "-C", str(source), "status", "--porcelain", "--untracked-files=no"], text=True):
        parser.error("tracked reference files have local changes")
    for item in baseline["files"]:
        if hashlib.sha256((source / item["path"]).read_bytes()).hexdigest() != item["sha256"]:
            parser.error(f"source hash mismatch: {item['path']}")
    sys.path[:0] = [str(source / "src"), str(source / "tests")]
    from semantic_operators.types import Boolean, Call, Region, Score, complement, make_answer
    from semantic_operators.errors import ProviderError, ProviderTimeout
    from semantic_operators.filtering import filter_items
    from semantic_operators.rerank import rerank, reweighted
    from semantic_operators.images import Image, frames, replace_images
    from imagekit import png

    class Synthetic:
        def __init__(self):
            self.seen = []

        def ask(self, state, questions):
            self.seen.append(state)
            fixture = state.get("document", state.get("item", state)) if isinstance(state, dict) else state
            if not isinstance(fixture, dict):
                fixture = {}
            if fixture.get("error") == "timeout":
                raise ProviderTimeout("synthetic", "offline timeout")
            if fixture.get("error"):
                raise ProviderError("synthetic", "offline failure")
            name, question = next(iter(questions.items()))
            if "provider_threshold" in fixture:
                question = dataclasses.replace(question, min_confidence=fixture["provider_threshold"])
            if isinstance(question, Boolean):
                p = fixture.get("p", 0.9)
                probabilities = {"true": p, "false": complement(p)}
                value = p > 0.5
            else:
                probabilities = dict(zip(question.levels, fixture.get("p", [0.0, 0.0, 1.0]), strict=True))
                value = sum(i * probabilities[level] for i, level in enumerate(question.levels))
            return {name: make_answer(question, value, probabilities,
                                      call=Call("synthetic", fixture.get("model", "model-a")))}

    def evidence(answer):
        if isinstance(answer, ProviderTimeout):
            return {"error": "timeout"}
        if isinstance(answer, ProviderError):
            return {"error": "provider"}
        return {"value": answer.value, "confidence": answer.confidence,
                "probabilities": answer.probabilities, "model": answer.call.model}

    def filter_expected(result):
        return {"models": sorted(result.models), "verdicts": [
            {"id": v.id, "outcome": v.outcome, "result": evidence(v.error if v.error is not None else v.answer)}
            for v in result.verdicts]}

    def ranking_expected(result):
        top = []
        for k, partial in [(0, False), (1, False), (10, False), (10, True)]:
            item = {"k": k, "allow_partial": partial}
            try:
                item["ids"] = [r.id for r in result.top(k, allow_partial=partial)]
            except ValueError:
                item["error"] = True
            top.append(item)
        return {"complete": result.complete, "models": sorted(result.models),
                "ranked": [{"id": r.id, "position": r.position, "score": r.score, "answer": evidence(r.answer)} for r in result.ranked],
                "unscored": [{"id": id, "result": evidence(item)} for id, item in result.unscored.items()], "top": top}

    filters = [
        ("all_outcomes", 0.8, None, [("yes", {"p": 0.95}), ("no", {"p": 0.05}), ("tie", {"p": 0.5}),
                                  ("low", {"p": 0.6}), ("bad", {"error": "provider"}), ("late", {"error": "timeout"})]),
        ("threshold_equality", 0.93, None, [("equal", {"p": 0.07}), ("below", {"p": 0.08})]),
        ("ties_and_false_at_zero_threshold", 0.0, None, [("tie", {"p": 0.5}), ("false", {"p": 0.0})]),
        ("context_wrapper", 0.0, {"policy": ["one", "two"]}, [("local-id", {"p": 0.9})]),
        ("empty", 0.8, None, []),
        ("reported_models_include_undecided", 0.8, None, [("a", {"p": 0.9}), ("b", {"p": 0.5, "model": "model-b"}), ("c", {"p": 0.1, "model": None})]),
    ]
    filter_cases = []
    for id, threshold, context, items in filters:
        provider = Synthetic()
        result = filter_items(provider, Boolean("match?", min_confidence=threshold), dict(items), context=context)
        filter_cases.append({"id": id, "threshold": threshold, "context": context, "items": items,
                             "expected": filter_expected(result), "expected_states": provider.seen})

    ranking_inputs = [
        ("descending_exact_ties", 0.0, None, [("a", {"p": [0.6, 0.4, 0.0]}), ("b", {"p": [0.0, 0.2, 0.8]}), ("c", {"p": [0.6, 0.4, 0.0]})], None),
        ("partial_and_provider_abstention", 0.0, None, [("a", {"p": [0.0, 0.0, 1.0]}), ("b", {"error": "provider"}), ("c", {"p": [0.2, 0.5, 0.3], "provider_threshold": 0.9})], None),
        ("mixed_models_decided", 0.0, None, [("a", {"p": [0.0, 0.0, 1.0]}), ("b", {"p": [1.0, 0.0, 0.0], "model": "model-b"})], None),
        ("mixed_models_undecided", 0.8, None, [("a", {"p": [0.0, 0.0, 1.0]}), ("b", {"p": [0.4, 0.3, 0.3], "model": "model-b"})], None),
        ("arbitrary_document_and_context", 0.0, ["background"], [("null", None), ("array", [1, 2]), ("bool", False)], None),
        ("empty", 0.0, None, [], None),
        ("weighted_order_changes", 0.0, None, [("a", {"p": [0.0, 0.8, 0.2]}), ("b", {"p": [0.5, 0.0, 0.5]})], [0.0, 1.0, 100.0]),
        ("weighted_mass_normalization", 0.0, None, [("rounded", {"p": [0.2, 0.8, 0.002]})], [0.0, 1.0, 2.0]),
        ("negative_weights", 0.0, None, [("a", {"p": [0.0, 0.8, 0.2]}), ("b", {"p": [0.5, 0.0, 0.5]})], [-10.0, -1.0, 2.0]),
        ("duplicate_weights_invalid", 0.0, None, [("a", {"p": [0.0, 0.8, 0.2]})], [0.0, 1.0, 1.0]),
    ]
    rank_cases = []
    for id, threshold, context, items, weights in ranking_inputs:
        provider = Synthetic()
        ranking = rerank(provider, Score("relevance?", ["none", "some", "full"], min_confidence=threshold),
                         query="synthetic query", candidates=dict(items), context=context)
        record = {"id": id, "threshold": threshold, "context": context, "items": items,
                  "expected": ranking_expected(ranking), "expected_states": provider.seen}
        if weights is not None:
            record["weights"] = weights
            try:
                record["expected_reweighted"] = ranking_expected(reweighted(ranking, weights))
            except ValueError:
                record["expected_reweight_error"] = True
        rank_cases.append(record)

    data = png(300, 400)
    images = [Image(data), Image(data)]
    frame_cases = []
    for id, indexes, times in [("plain", [0, 1], None), ("timestamps_preserved", [1, 0], [-1.0, 2.0]),
                               ("empty_invalid", [], None), ("length_invalid", [0], [1, 2])]:
        record = {"id": id, "images": indexes, "times": times}
        try:
            result = replace_images(frames([images[i] for i in indexes], times))
            for item in result:
                item["frame"] = f"frame-{next(i for i, image in enumerate(images) if image.placeholder == item['frame'])}"
            record["expected"] = result
        except ValueError:
            record["expected_error"] = True
        frame_cases.append(record)

    region_cases = []
    for id, edges, fraction, size in [
        ("ordinary", [0.2, 0.6, 0.1, 0.5], 0.0, [1000, 500]),
        ("padding_clips", [0.2, 0.6, 0.1, 0.5], 1.0, [1000, 500]),
        ("pixel_ties_even", [0.125, 0.625, 0.375, 0.875], 0.0, [4, 4]),
        ("negative_padding_shrinks", [0.2, 0.6, 0.1, 0.5], -0.25, [1000, 500]),
        ("over_shrinking_invalid", [0.2, 0.6, 0.1, 0.5], -1.0, [1000, 500]),
    ]:
        region = Region(*edges)
        record = {"id": id, "edges": edges, "padding": fraction, "size": size,
                  "expected_pixels": region.pixels(*size), "expected_center": region.center}
        try:
            record["expected_padded"] = dataclasses.asdict(region.padded(fraction))
        except ValueError:
            record["expected_padding_error"] = True
        region_cases.append(record)
    source_paths = {"src/semantic_operators/types.py", "src/semantic_operators/_each.py", "src/semantic_operators/filtering.py",
                    "src/semantic_operators/rerank.py", "src/semantic_operators/images.py", "tests/imagekit.py"}
    document = {"fixture_version": 1, "reference_revision": revision,
                "python_version": platform.python_version(), "python_implementation": platform.python_implementation(),
                "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "reference_files": [item for item in baseline["files"] if item["path"] in source_paths],
                "scope": "Synthetic offline reference execution; no SDK, network, credentials, or model calls.",
                "filter_cases": filter_cases, "ranking_cases": rank_cases,
                "image_base64": base64.b64encode(data).decode("ascii"), "frame_cases": frame_cases, "region_cases": region_cases}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(document, indent=2, allow_nan=False) + "\n")
    print(f"Wrote {len(filter_cases)+len(rank_cases)+len(frame_cases)+len(region_cases)} independent Python recipe/frame/region cases")


if __name__ == "__main__":
    main()
