<!-- WARNING: This file is auto-generated (cargo run -p seal_dev generate-all). Edit the doc comments in 'crates/seal_cli/src/lib.rs' if you want to change anything here. -->

# CLI Reference

## seal

An extremely fast release management tool.

<h3 class="cli-reference">Usage</h3>

```
seal [OPTIONS] <COMMAND>
```

<h3 class="cli-reference">Commands</h3>

<dl class="cli-reference"><dt><a href="#seal-self"><code>seal self</code></a></dt><dd><p>Manage the seal executable</p></dd>
<dt><a href="#seal-validate"><code>seal validate</code></a></dt><dd><p>Validate project configuration and structure</p></dd>
<dt><a href="#seal-bump"><code>seal bump</code></a></dt><dd><p>Bump version and create release branch</p></dd>
<dt><a href="#seal-help"><code>seal help</code></a></dt><dd><p>Display documentation for a command</p></dd>
</dl>

## seal self

Manage the seal executable

<h3 class="cli-reference">Usage</h3>

```
seal self [OPTIONS] <COMMAND>
```

<h3 class="cli-reference">Commands</h3>

<dl class="cli-reference"><dt><a href="#seal-self-version"><code>seal self version</code></a></dt><dd><p>Display seal's version</p></dd>
</dl>

### seal self version

Display seal's version

<h3 class="cli-reference">Usage</h3>

```
seal self version [OPTIONS]
```

<h3 class="cli-reference">Options</h3>

<dl class="cli-reference"><dt id="seal-self-version--help"><a href="#seal-self-version--help"><code>--help</code></a>, <code>-h</code></dt><dd><p>Display the concise help for this command</p>
</dd><dt id="seal-self-version--no-progress"><a href="#seal-self-version--no-progress"><code>--no-progress</code></a></dt><dd><p>Hide all progress outputs.</p>
<p>For example, spinners or progress bars.</p>
</dd><dt id="seal-self-version--output-format"><a href="#seal-self-version--output-format"><code>--output-format</code></a> <i>output-format</i></dt><dt id="seal-self-version--quiet"><a href="#seal-self-version--quiet"><code>--quiet</code></a>, <code>-q</code></dt><dd><p>Use quiet output.</p>
<p>Repeating this option, e.g., <code>-qq</code>, will enable a silent mode in which seal will write no output to stdout.</p>
</dd><dt id="seal-self-version--short"><a href="#seal-self-version--short"><code>--short</code></a></dt><dd><p>Only print the version</p>
</dd><dt id="seal-self-version--verbose"><a href="#seal-self-version--verbose"><code>--verbose</code></a>, <code>-v</code></dt><dd><p>Use verbose output</p>
</dd></dl>

## seal validate

Validate project configuration and structure

<h3 class="cli-reference">Usage</h3>

```
seal validate [OPTIONS] <COMMAND>
```

<h3 class="cli-reference">Commands</h3>

<dl class="cli-reference"><dt><a href="#seal-validate-config"><code>seal validate config</code></a></dt><dd><p>Validate workspace configuration file</p></dd>
<dt><a href="#seal-validate-project"><code>seal validate project</code></a></dt><dd><p>Validate full project workspace including members</p></dd>
</dl>

### seal validate config

Validate workspace configuration file

If no config path is provided, discovers seal.toml in the current directory.

<h3 class="cli-reference">Usage</h3>

```
seal validate config [OPTIONS]
```

<h3 class="cli-reference">Options</h3>

<dl class="cli-reference"><dt id="seal-validate-config--config-file"><a href="#seal-validate-config--config-file"><code>--config-file</code></a> <i>config-file</i></dt><dd><p>Path to the config file (seal.toml)</p>
</dd><dt id="seal-validate-config--help"><a href="#seal-validate-config--help"><code>--help</code></a>, <code>-h</code></dt><dd><p>Display the concise help for this command</p>
</dd><dt id="seal-validate-config--no-progress"><a href="#seal-validate-config--no-progress"><code>--no-progress</code></a></dt><dd><p>Hide all progress outputs.</p>
<p>For example, spinners or progress bars.</p>
</dd><dt id="seal-validate-config--quiet"><a href="#seal-validate-config--quiet"><code>--quiet</code></a>, <code>-q</code></dt><dd><p>Use quiet output.</p>
<p>Repeating this option, e.g., <code>-qq</code>, will enable a silent mode in which seal will write no output to stdout.</p>
</dd><dt id="seal-validate-config--verbose"><a href="#seal-validate-config--verbose"><code>--verbose</code></a>, <code>-v</code></dt><dd><p>Use verbose output</p>
</dd></dl>

### seal validate project

Validate full project workspace including members

If no project path is provided, uses the current directory.

<h3 class="cli-reference">Usage</h3>

```
seal validate project [OPTIONS]
```

<h3 class="cli-reference">Options</h3>

<dl class="cli-reference"><dt id="seal-validate-project--help"><a href="#seal-validate-project--help"><code>--help</code></a>, <code>-h</code></dt><dd><p>Display the concise help for this command</p>
</dd><dt id="seal-validate-project--no-progress"><a href="#seal-validate-project--no-progress"><code>--no-progress</code></a></dt><dd><p>Hide all progress outputs.</p>
<p>For example, spinners or progress bars.</p>
</dd><dt id="seal-validate-project--project"><a href="#seal-validate-project--project"><code>--project</code></a>, <code>-p</code> <i>project</i></dt><dd><p>Path to the project directory</p>
</dd><dt id="seal-validate-project--quiet"><a href="#seal-validate-project--quiet"><code>--quiet</code></a>, <code>-q</code></dt><dd><p>Use quiet output.</p>
<p>Repeating this option, e.g., <code>-qq</code>, will enable a silent mode in which seal will write no output to stdout.</p>
</dd><dt id="seal-validate-project--verbose"><a href="#seal-validate-project--verbose"><code>--verbose</code></a>, <code>-v</code></dt><dd><p>Use verbose output</p>
</dd></dl>

## seal bump

Bump version and create release branch

<h3 class="cli-reference">Usage</h3>

```
seal bump [OPTIONS] <VERSION>
```

<h3 class="cli-reference">Arguments</h3>

<dl class="cli-reference"><dt id="seal-bump--version"><a href="#seal-bump--version"><code>VERSION</code></a></dt><dd><p>Version bump to perform (e.g., 'major', 'minor', 'patch', 'alpha', 'major-beta', or '1.2.3')</p>
</dd></dl>

<h3 class="cli-reference">Options</h3>

<dl class="cli-reference"><dt id="seal-bump--dry-run"><a href="#seal-bump--dry-run"><code>--dry-run</code></a></dt><dd><p>Show what would be done without making any changes</p>
</dd><dt id="seal-bump--help"><a href="#seal-bump--help"><code>--help</code></a>, <code>-h</code></dt><dd><p>Display the concise help for this command</p>
</dd><dt id="seal-bump--no-progress"><a href="#seal-bump--no-progress"><code>--no-progress</code></a></dt><dd><p>Hide all progress outputs.</p>
<p>For example, spinners or progress bars.</p>
</dd><dt id="seal-bump--quiet"><a href="#seal-bump--quiet"><code>--quiet</code></a>, <code>-q</code></dt><dd><p>Use quiet output.</p>
<p>Repeating this option, e.g., <code>-qq</code>, will enable a silent mode in which seal will write no output to stdout.</p>
</dd><dt id="seal-bump--verbose"><a href="#seal-bump--verbose"><code>--verbose</code></a>, <code>-v</code></dt><dd><p>Use verbose output</p>
</dd></dl>

## seal help

Display documentation for a command

<h3 class="cli-reference">Usage</h3>

```
seal help [OPTIONS] [COMMAND]...
```

