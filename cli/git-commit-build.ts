async function getGitCommitHash(): Promise<string> {
    try {
        const proc = Bun.spawn(["git", "rev-parse", "--short", "HEAD"]);
        const text = await new Response(proc.stdout).text();
        return text.trim();
    } catch {
        return "unknown";
    }
}

Bun.file("version").write(await getGitCommitHash());

export {};