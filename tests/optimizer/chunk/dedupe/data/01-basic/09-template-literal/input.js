import { dedupe } from "oveo";

dedupe({ a: `ab${c}d` });
dedupe({ a: `ab${c}d` });
dedupe({ a: `ad${c}b` });
