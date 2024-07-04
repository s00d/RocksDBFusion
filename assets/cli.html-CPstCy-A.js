import{_ as s}from"./plugin-vue_export-helper-DlAUqK2U.js";import{c as a,a as e,d as t,o as h,r as l}from"./app-C-HbIo8n.js";const n={},r=t(`<h2 id="rocksdb-cli-client" tabindex="-1"><a class="header-anchor" href="#rocksdb-cli-client"><span>RocksDB CLI Client</span></a></h2><p>A simple CLI client to interact with a RocksDB database. This client allows you to perform various operations such as storing, retrieving, deleting, and merging key-value pairs. It also supports advanced operations like managing column families and transactions.</p><h2 id="features" tabindex="-1"><a class="header-anchor" href="#features"><span>Features</span></a></h2><ul><li>Store a key-value pair in the database</li><li>Retrieve the value of a key from the database</li><li>Delete a key from the database</li><li>Merge a value with an existing key</li><li>List all column families in the database</li><li>Create and drop column families</li><li>Compact the database within a range</li><li>Support for transactions (begin, commit, rollback)</li></ul><h2 id="prerequisites" tabindex="-1"><a class="header-anchor" href="#prerequisites"><span>Prerequisites</span></a></h2><ul><li><a href="https://www.rust-lang.org/tools/install" target="_blank" rel="noopener noreferrer">Rust</a> (only for building from source)</li></ul><h2 id="getting-started" tabindex="-1"><a class="header-anchor" href="#getting-started"><span>Getting Started</span></a></h2><h3 id="download-the-application" tabindex="-1"><a class="header-anchor" href="#download-the-application"><span>Download the Application</span></a></h3><p>Pre-built versions of the application are available in the <a href="https://github.com/s00d/RocksDBFusion/releases" target="_blank" rel="noopener noreferrer">Releases</a>. You can download the appropriate version for your operating system and run the application directly.</p><p>For example, you can download the specific version from <a href="https://github.com/s00d/RocksDBFusion/releases/tag/rocksdb-cli-v0.1.1" target="_blank" rel="noopener noreferrer">this link</a>.</p><h3 id="extract-the-downloaded-archive" tabindex="-1"><a class="header-anchor" href="#extract-the-downloaded-archive"><span>Extract the Downloaded Archive</span></a></h3><p>After downloading the appropriate archive for your operating system, extract it:</p><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">tar</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> -xzf</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> rocksdb_cli-</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">platfor</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">m</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">.tar.gz</span><span style="--shiki-light:#6A737D;--shiki-dark:#7F848E;--shiki-light-font-style:inherit;--shiki-dark-font-style:italic;">  # For Linux and macOS</span></span>
<span class="line"><span style="--shiki-light:#6A737D;--shiki-dark:#7F848E;--shiki-light-font-style:inherit;--shiki-dark-font-style:italic;"># or</span></span>
<span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">7z</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> x</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> rocksdb_cli-</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">platfor</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">m</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">.zip</span><span style="--shiki-light:#6A737D;--shiki-dark:#7F848E;--shiki-light-font-style:inherit;--shiki-dark-font-style:italic;">         # For Windows</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="install-with-homebrew-macos" tabindex="-1"><a class="header-anchor" href="#install-with-homebrew-macos"><span>Install with Homebrew (macOS)</span></a></h3><p>You can also install RocksDB CLI Client using Homebrew on macOS. First, make sure to tap the repository:</p><div class="language-sh line-numbers-mode" data-highlighter="shiki" data-ext="sh" data-title="sh" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">brew</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> tap</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> s00d/rocksdbserver</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><p>Then, install the RocksDB CLI Client:</p><div class="language-sh line-numbers-mode" data-highlighter="shiki" data-ext="sh" data-title="sh" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">brew</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> install</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> rocksdb_cli</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h3 id="run-the-application" tabindex="-1"><a class="header-anchor" href="#run-the-application"><span>Run the Application</span></a></h3><p>Navigate to the directory where the binary is extracted and run the application:</p><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> ...</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h3 id="macos-sign" tabindex="-1"><a class="header-anchor" href="#macos-sign"><span>macOS Sign</span></a></h3><p>If you are on macOS, you may need to sign the application before running it. Here are the steps:</p><ol><li><p>Make the binary executable:</p><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">chmod</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> +x</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> ./rocksdb_cli</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div></li><li><p>Clear extended attributes and sign the binary:</p><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">xattr</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> -cr</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> ./rocksdb_cli</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;"> &amp;&amp; </span><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">codesign</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --force</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --deep</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --sign</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> -</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> ./rocksdb_cli</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div></li></ol><h3 id="command-usage" tabindex="-1"><a class="header-anchor" href="#command-usage"><span>Command Usage</span></a></h3><h4 id="common-options" tabindex="-1"><a class="header-anchor" href="#common-options"><span>Common Options</span></a></h4><ul><li><code>--host</code>: Server host (default: <code>127.0.0.1</code>)</li><li><code>--port</code>: Server port (default: <code>12345</code>)</li></ul><h4 id="store-a-key-value-pair" tabindex="-1"><a class="header-anchor" href="#store-a-key-value-pair"><span>Store a Key-Value Pair</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> put</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">ke</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">y</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">valu</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">e</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="retrieve-the-value-of-a-key" tabindex="-1"><a class="header-anchor" href="#retrieve-the-value-of-a-key"><span>Retrieve the Value of a Key</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> get</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">ke</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">y</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="delete-a-key" tabindex="-1"><a class="header-anchor" href="#delete-a-key"><span>Delete a Key</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> delete</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">ke</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">y</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="merge-a-value-with-an-existing-key" tabindex="-1"><a class="header-anchor" href="#merge-a-value-with-an-existing-key"><span>Merge a Value with an Existing Key</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> merge</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">ke</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">y</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">valu</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">e</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="list-all-column-families" tabindex="-1"><a class="header-anchor" href="#list-all-column-families"><span>List All Column Families</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> list_column_families</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="create-a-new-column-family" tabindex="-1"><a class="header-anchor" href="#create-a-new-column-family"><span>Create a New Column Family</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> create_column_family</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">nam</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">e</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="drop-an-existing-column-family" tabindex="-1"><a class="header-anchor" href="#drop-an-existing-column-family"><span>Drop an Existing Column Family</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> drop_column_family</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;"> &lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">nam</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">e</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="compact-the-database-within-a-range" tabindex="-1"><a class="header-anchor" href="#compact-the-database-within-a-range"><span>Compact the Database Within a Range</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> compact_range</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;"> [--start </span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">start_ke</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">y</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">]</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;"> [--end </span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&lt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">end_ke</span><span style="--shiki-light:#24292E;--shiki-dark:#ABB2BF;">y</span><span style="--shiki-light:#D73A49;--shiki-dark:#ABB2BF;">&gt;</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;">]</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="begin-a-new-transaction" tabindex="-1"><a class="header-anchor" href="#begin-a-new-transaction"><span>Begin a New Transaction</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> begin_transaction</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="commit-a-transaction" tabindex="-1"><a class="header-anchor" href="#commit-a-transaction"><span>Commit a Transaction</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> commit_transaction</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h4 id="rollback-a-transaction" tabindex="-1"><a class="header-anchor" href="#rollback-a-transaction"><span>Rollback a Transaction</span></a></h4><div class="language-bash line-numbers-mode" data-highlighter="shiki" data-ext="bash" data-title="bash" style="--shiki-light:#24292e;--shiki-dark:#abb2bf;--shiki-light-bg:#fff;--shiki-dark-bg:#282c34;"><pre class="shiki shiki-themes github-light one-dark-pro vp-code"><code><span class="line"><span style="--shiki-light:#6F42C1;--shiki-dark:#61AFEF;">./rocksdb_cli</span><span style="--shiki-light:#032F62;--shiki-dark:#98C379;"> rollback_transaction</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --host</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 127.0.0.1</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> --port</span><span style="--shiki-light:#005CC5;--shiki-dark:#D19A66;"> 12345</span></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div></div></div><h3 id="detailed-workflow" tabindex="-1"><a class="header-anchor" href="#detailed-workflow"><span>Detailed Workflow</span></a></h3><p>Below is a detailed workflow of how the client interacts with the RocksDB server:</p>`,51);function k(d,p){const i=l("Mermaid");return h(),a("div",null,[r,e(i,{id:"mermaid-186",code:"eJxtkFEKwjAMht89RS7gBfrgg9uLICibHqDUIGVbWptu4O1NN4WW2pdC/j/5kp/xNSMZbK1+Bj3tQJ7XIVpjvaYId8ZQFZvzqap1M8ejNgPSo9JuxjeOCE20jupOZwZujz2GRVirnKj7w0E4SgYTNG6a9HewFEXKcAquwRlkLmyZQezFBgouHgluzRV6YWNcGwpLIuRrKehlEHQpLd78hV4jfjvlLRUiP6JDg3ZB+dk7YvxzxZYHxjlQadsySakpaC37Ub+TYR7j7gN2kp/Q"})])}const g=s(n,[["render",k],["__file","cli.html.vue"]]),b=JSON.parse('{"path":"/cli.html","title":"","lang":"en-US","frontmatter":{"lang":"en-US","icon":"fas fa-terminal","order":4},"headers":[{"level":2,"title":"RocksDB CLI Client","slug":"rocksdb-cli-client","link":"#rocksdb-cli-client","children":[]},{"level":2,"title":"Features","slug":"features","link":"#features","children":[]},{"level":2,"title":"Prerequisites","slug":"prerequisites","link":"#prerequisites","children":[]},{"level":2,"title":"Getting Started","slug":"getting-started","link":"#getting-started","children":[{"level":3,"title":"Download the Application","slug":"download-the-application","link":"#download-the-application","children":[]},{"level":3,"title":"Extract the Downloaded Archive","slug":"extract-the-downloaded-archive","link":"#extract-the-downloaded-archive","children":[]},{"level":3,"title":"Install with Homebrew (macOS)","slug":"install-with-homebrew-macos","link":"#install-with-homebrew-macos","children":[]},{"level":3,"title":"Run the Application","slug":"run-the-application","link":"#run-the-application","children":[]},{"level":3,"title":"macOS Sign","slug":"macos-sign","link":"#macos-sign","children":[]},{"level":3,"title":"Command Usage","slug":"command-usage","link":"#command-usage","children":[]},{"level":3,"title":"Detailed Workflow","slug":"detailed-workflow","link":"#detailed-workflow","children":[]}]}],"git":{"createdTime":1720083201000,"updatedTime":1720083201000,"contributors":[{"name":"Pavel Kuzmin","email":"Virus191288@gmail.com","commits":1}]},"readingTime":{"minutes":1.72,"words":516},"filePathRelative":"cli.md","localizedDate":"July 4, 2024","excerpt":"<h2>RocksDB CLI Client</h2>\\n<p>A simple CLI client to interact with a RocksDB database. This client allows you to perform various operations such as storing, retrieving, deleting, and merging key-value pairs. It also supports advanced operations like managing column families and transactions.</p>\\n<h2>Features</h2>"}');export{g as comp,b as data};