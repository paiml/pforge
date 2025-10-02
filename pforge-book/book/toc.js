// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="title-page.html">pforge: EXTREME TDD for MCP Servers</a></li><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded "><a href="ch01-00-pforge-vs-pmcp.html"><strong aria-hidden="true">1.</strong> Chapter 1: pforge vs pmcp (rust-mcp-sdk)</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch01-01-when-pforge.html"><strong aria-hidden="true">1.1.</strong> When to Use pforge</a></li><li class="chapter-item "><a href="ch01-02-when-pmcp.html"><strong aria-hidden="true">1.2.</strong> When to Use pmcp</a></li><li class="chapter-item "><a href="ch01-03-comparison.html"><strong aria-hidden="true">1.3.</strong> Side-by-Side Comparison</a></li><li class="chapter-item "><a href="ch01-04-migration.html"><strong aria-hidden="true">1.4.</strong> Migration Between Them</a></li><li class="chapter-item "><a href="ch01-05-architecture-pmcp.html"><strong aria-hidden="true">1.5.</strong> Architecture: How pforge Uses pmcp</a></li></ol></li><li class="chapter-item expanded "><a href="ch02-00-quick-start.html"><strong aria-hidden="true">2.</strong> Chapter 2: Quick Start</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch02-01-installation.html"><strong aria-hidden="true">2.1.</strong> Installation</a></li><li class="chapter-item "><a href="ch02-02-first-server.html"><strong aria-hidden="true">2.2.</strong> Your First Server (5 Minutes)</a></li><li class="chapter-item "><a href="ch02-03-testing.html"><strong aria-hidden="true">2.3.</strong> Testing Your Server</a></li></ol></li><li class="chapter-item expanded "><a href="ch03-00-calculator.html"><strong aria-hidden="true">3.</strong> Chapter 3: Calculator Server</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch03-01-yaml-config.html"><strong aria-hidden="true">3.1.</strong> YAML Configuration</a></li><li class="chapter-item "><a href="ch03-02-handler.html"><strong aria-hidden="true">3.2.</strong> Rust Handler Implementation</a></li><li class="chapter-item "><a href="ch03-03-tests.html"><strong aria-hidden="true">3.3.</strong> Unit Tests</a></li><li class="chapter-item "><a href="ch03-04-running.html"><strong aria-hidden="true">3.4.</strong> Running the Server</a></li></ol></li><li class="chapter-item expanded "><a href="ch04-00-file-ops.html"><strong aria-hidden="true">4.</strong> Chapter 4: File Operations Server</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch04-01-cli-wrappers.html"><strong aria-hidden="true">4.1.</strong> CLI Tool Wrappers</a></li><li class="chapter-item "><a href="ch04-02-streaming.html"><strong aria-hidden="true">4.2.</strong> Streaming Output</a></li><li class="chapter-item "><a href="ch04-03-integration-tests.html"><strong aria-hidden="true">4.3.</strong> Integration Tests</a></li></ol></li><li class="chapter-item expanded "><a href="ch05-00-github-api.html"><strong aria-hidden="true">5.</strong> Chapter 5: GitHub API Server</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch05-01-http-config.html"><strong aria-hidden="true">5.1.</strong> HTTP Tool Configuration</a></li><li class="chapter-item "><a href="ch05-02-authentication.html"><strong aria-hidden="true">5.2.</strong> Authentication</a></li><li class="chapter-item "><a href="ch05-03-error-handling.html"><strong aria-hidden="true">5.3.</strong> Error Handling</a></li></ol></li><li class="chapter-item expanded "><a href="ch06-00-data-pipeline.html"><strong aria-hidden="true">6.</strong> Chapter 6: Data Pipeline Server</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch06-01-composition.html"><strong aria-hidden="true">6.1.</strong> Pipeline Composition</a></li><li class="chapter-item "><a href="ch06-02-conditionals.html"><strong aria-hidden="true">6.2.</strong> Conditional Execution</a></li><li class="chapter-item "><a href="ch06-03-state.html"><strong aria-hidden="true">6.3.</strong> State Management</a></li></ol></li><li class="chapter-item expanded "><a href="ch07-00-five-minute-cycle.html"><strong aria-hidden="true">7.</strong> Chapter 7: The 5-Minute TDD Cycle</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch07-01-red.html"><strong aria-hidden="true">7.1.</strong> RED: Write Failing Test</a></li><li class="chapter-item "><a href="ch07-02-green.html"><strong aria-hidden="true">7.2.</strong> GREEN: Minimum Code</a></li><li class="chapter-item "><a href="ch07-03-refactor.html"><strong aria-hidden="true">7.3.</strong> REFACTOR: Clean Up</a></li><li class="chapter-item "><a href="ch07-04-commit.html"><strong aria-hidden="true">7.4.</strong> COMMIT: Quality Gates</a></li></ol></li><li class="chapter-item expanded "><a href="ch08-00-quality-gates.html"><strong aria-hidden="true">8.</strong> Chapter 8: Quality Gates</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch08-01-pre-commit.html"><strong aria-hidden="true">8.1.</strong> Pre-Commit Hooks</a></li><li class="chapter-item "><a href="ch08-02-pmat.html"><strong aria-hidden="true">8.2.</strong> PMAT Integration</a></li><li class="chapter-item "><a href="ch08-03-complexity.html"><strong aria-hidden="true">8.3.</strong> Complexity Limits</a></li><li class="chapter-item "><a href="ch08-04-coverage.html"><strong aria-hidden="true">8.4.</strong> Coverage Requirements</a></li></ol></li><li class="chapter-item expanded "><a href="ch09-00-testing-strategies.html"><strong aria-hidden="true">9.</strong> Chapter 9: Testing Strategies</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch09-01-unit-testing.html"><strong aria-hidden="true">9.1.</strong> Unit Testing</a></li><li class="chapter-item "><a href="ch09-02-integration-testing.html"><strong aria-hidden="true">9.2.</strong> Integration Testing</a></li><li class="chapter-item "><a href="ch09-03-property-testing.html"><strong aria-hidden="true">9.3.</strong> Property-Based Testing</a></li><li class="chapter-item "><a href="ch09-04-mutation-testing.html"><strong aria-hidden="true">9.4.</strong> Mutation Testing</a></li></ol></li><li class="chapter-item expanded "><a href="ch10-00-state-management.html"><strong aria-hidden="true">10.</strong> Chapter 10: State Management</a></li><li class="chapter-item expanded "><a href="ch11-00-fault-tolerance.html"><strong aria-hidden="true">11.</strong> Chapter 11: Fault Tolerance</a></li><li class="chapter-item expanded "><a href="ch12-00-middleware.html"><strong aria-hidden="true">12.</strong> Chapter 12: Middleware</a></li><li class="chapter-item expanded "><a href="ch13-00-resources-prompts.html"><strong aria-hidden="true">13.</strong> Chapter 13: Resources &amp; Prompts</a></li><li class="chapter-item expanded "><a href="ch14-00-performance.html"><strong aria-hidden="true">14.</strong> Chapter 14: Performance Targets</a></li><li class="chapter-item expanded "><a href="ch15-00-benchmarking.html"><strong aria-hidden="true">15.</strong> Chapter 15: Benchmarking</a></li><li class="chapter-item expanded "><a href="ch16-00-codegen.html"><strong aria-hidden="true">16.</strong> Chapter 16: Code Generation</a></li><li class="chapter-item expanded "><a href="ch17-00-publishing-crates.html"><strong aria-hidden="true">17.</strong> Chapter 17: Publishing to Crates.io</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="ch17-01-preparing.html"><strong aria-hidden="true">17.1.</strong> Preparing Your Crate</a></li><li class="chapter-item "><a href="ch17-02-versioning.html"><strong aria-hidden="true">17.2.</strong> Version Management</a></li><li class="chapter-item "><a href="ch17-03-documentation.html"><strong aria-hidden="true">17.3.</strong> Documentation</a></li><li class="chapter-item "><a href="ch17-04-publishing.html"><strong aria-hidden="true">17.4.</strong> Publishing Process</a></li></ol></li><li class="chapter-item expanded "><a href="ch18-00-cicd.html"><strong aria-hidden="true">18.</strong> Chapter 18: CI/CD Pipeline</a></li><li class="chapter-item expanded "><a href="ch19-00-bridges.html"><strong aria-hidden="true">19.</strong> Chapter 19: Multi-Language Bridges</a></li><li class="chapter-item expanded "><a href="appendix-a-config-reference.html"><strong aria-hidden="true">20.</strong> Appendix A: Complete Configuration Reference</a></li><li class="chapter-item expanded "><a href="appendix-b-api-docs.html"><strong aria-hidden="true">21.</strong> Appendix B: API Documentation</a></li><li class="chapter-item expanded "><a href="appendix-c-troubleshooting.html"><strong aria-hidden="true">22.</strong> Appendix C: Troubleshooting</a></li><li class="chapter-item expanded "><a href="appendix-d-contributing.html"><strong aria-hidden="true">23.</strong> Appendix D: Contributing</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
