"""Legacy report compatibility package.

Canonical package: atlasctl.reporting.
"""

from ..cli.registry import domain_payload


def run(ctx):
    return domain_payload(ctx, "report")
