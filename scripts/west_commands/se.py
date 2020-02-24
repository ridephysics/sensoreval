from west.commands import WestCommand
from west import log
from west import util
import os
import subprocess
import shutil

class Se(WestCommand):
    def __init__(self):
        super().__init__(
            'se',
            'build sensoreval tools',
            None)

        self.top_dir = util.west_topdir()
        self.build_dir = os.path.join(self.top_dir, 'build/sensoreval')
        self.source_dir = os.path.join(self.top_dir, 'sensoreval')

    def do_add_parser(self, parser_adder):
        parser = parser_adder.add_parser(self.name,
                                         help=self.help,
                                         description=self.description)

        parser.add_argument('action', help='action to run (build, clean)')
        parser.add_argument('--release', help='compile release build',
            action='store_true', default=False)

        return parser

    def run_cmd(self, args, env={}):
        newenv = os.environ.copy()
        newenv.update(env)

        p = subprocess.Popen(args, cwd=self.build_dir, env=newenv)
        p.communicate()

        if p.returncode:
            raise subprocess.CalledProcessError(p.returncode, args)

    def run_cmake(self, program_args):
        os.makedirs(self.build_dir, exist_ok=True)

        args = [
            'cmake',
            '-G', 'Ninja',
            '-D', 'EXTERNAL_DIR=' + os.path.join(self.top_dir, 'external'),
            '-D', 'COMPONENTS_DIR=' + os.path.join(self.top_dir, 'components'),
            '-D', 'BMP280_DIR=' + os.path.join(self.top_dir, 'external/bmp280'),
            '-D', 'BUILD_SHARED_LIBS=OFF',
        ]

        args.append('-D')
        if program_args.release:
            args.append('CMAKE_BUILD_TYPE=Release')
        else:
            args.append('CMAKE_BUILD_TYPE=Debug')

        args.append(self.source_dir)

        env = {
            'CFLAGS': '-fdiagnostics-color=always',
            'CXXFLAGS': '-fdiagnostics-color=always'
        }

        self.run_cmd(args, env=env)

    def do_run(self, args, unknown_args):
        if args.action == 'clean':
            if os.path.exists(self.build_dir):
                shutil.rmtree(self.build_dir)
            return

        if args.action == 'build':
            env = {
                'DESTDIR': os.path.join(self.build_dir, 'install')
            }

            self.run_cmake(args)
            self.run_cmd([
                'ninja', 'videorenderer'
            ], env=env)
            return

        else:
            raise Exception('unsupported action: %s' % (args.action))
