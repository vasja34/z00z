import edu.mit.csail.sdg.alloy4.A4Reporter;
import edu.mit.csail.sdg.alloy4.Err;
import edu.mit.csail.sdg.ast.Command;
import edu.mit.csail.sdg.parser.CompModule;
import edu.mit.csail.sdg.parser.CompUtil;
import edu.mit.csail.sdg.translator.A4Options;
import edu.mit.csail.sdg.translator.A4Solution;
import edu.mit.csail.sdg.translator.TranslateAlloyToKodkod;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;

public final class Z00ZAlloyHeadless {
    private Z00ZAlloyHeadless() {}

    private static void usage() {
        System.out.println("Usage: Z00ZAlloyHeadless <model.als> [more-models.als]");
        System.out.println("Runs every Alloy command in each model and returns non-zero on counterexample or parse failure.");
    }

    private static boolean commandPassed(Command command, boolean satisfiable) {
        if (command.expects == 0 || command.expects == 1) {
            return (satisfiable ? 1 : 0) == command.expects;
        }
        if (command.check) {
            return !satisfiable;
        }
        return satisfiable;
    }

    private static int runModel(Path modelPath) throws Err {
        if (!Files.isRegularFile(modelPath)) {
            System.err.printf("[z00z-alloy] ERROR %s is not a regular file%n", modelPath);
            return 2;
        }

        CompModule module = CompUtil.parseEverything_fromFile(new A4Reporter() {}, new HashMap<>(), modelPath.toString());
        if (module.getAllCommands().isEmpty()) {
            System.err.printf("[z00z-alloy] ERROR %s has no Alloy commands%n", modelPath);
            return 2;
        }

        A4Options options = new A4Options();
        int failures = 0;

        for (Command command : module.getAllCommands()) {
            A4Solution solution =
                TranslateAlloyToKodkod.execute_command(new A4Reporter() {}, module.getAllReachableSigs(), command, options);
            boolean satisfiable = solution.satisfiable();
            boolean passed = commandPassed(command, satisfiable);
            String kind = command.check ? "check" : "run";
            String verdict = passed ? "PASS" : "FAIL";
            String sat = satisfiable ? "sat" : "unsat";
            String label = command.label == null || command.label.isBlank() ? "<anonymous>" : command.label;
            System.out.printf("[z00z-alloy] %s %s :: %s :: %s%n", verdict, kind, label, sat);
            if (!passed) {
                failures += 1;
            }
        }

        return failures == 0 ? 0 : 1;
    }

    public static void main(String[] args) {
        if (args.length == 0 || "--help".equals(args[0]) || "-h".equals(args[0])) {
            usage();
            System.exit(args.length == 0 ? 2 : 0);
        }

        int exitCode = 0;
        for (String arg : args) {
            try {
                int modelExit = runModel(Path.of(arg));
                if (modelExit != 0 && exitCode == 0) {
                    exitCode = modelExit;
                }
            } catch (Err err) {
                System.err.printf("[z00z-alloy] ERROR %s :: %s%n", arg, err.getMessage());
                if (exitCode == 0) {
                    exitCode = 2;
                }
            }
        }
        System.exit(exitCode);
    }
}
