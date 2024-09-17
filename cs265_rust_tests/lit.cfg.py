import lit.formats
import os
# config.test_source_root = os.path.dirname(__file__)

# right now when e2e is called, this script thought that its in build/e2e-tests

config.name = "cs265 end-to-end testsuite"
config.test_format = lit.formats.ShTest(True)

config.suffixes = ['.bril']
