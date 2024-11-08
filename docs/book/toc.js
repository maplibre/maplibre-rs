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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="supported-platforms.html">Supported Platforms</a></li><li class="chapter-item expanded affix "><a href="developer-log.html">Developer Log</a></li><li class="chapter-item expanded "><a href="user-guide/index.html"><strong aria-hidden="true">1.</strong> User Guide</a></li><li class="chapter-item expanded "><a href="development-guide/index.html"><strong aria-hidden="true">2.</strong> Development Guide</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="development-guide/how-to-run.html"><strong aria-hidden="true">2.1.</strong> How to Run Demos</a></li><li class="chapter-item expanded "><a href="development-guide/building-libraries.html"><strong aria-hidden="true">2.2.</strong> Building Libraries</a></li><li class="chapter-item expanded "><a href="development-guide/debugging.html"><strong aria-hidden="true">2.3.</strong> Debugging</a></li></ol></li><li class="chapter-item expanded "><a href="development-documents/index.html"><strong aria-hidden="true">3.</strong> Development Documents</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="development-documents/architecture.html"><strong aria-hidden="true">3.1.</strong> Architecture</a></li><li class="chapter-item expanded "><a href="development-documents/design.html"><strong aria-hidden="true">3.2.</strong> Design</a></li><li class="chapter-item expanded "><a href="development-documents/caching.html"><strong aria-hidden="true">3.3.</strong> Caching</a></li><li class="chapter-item expanded "><a href="development-documents/stencil-masking.html"><strong aria-hidden="true">3.4.</strong> Stencil Masking</a></li><li class="chapter-item expanded "><a href="development-documents/font-rendering.html"><strong aria-hidden="true">3.5.</strong> Font Rendering</a></li><li class="chapter-item expanded "><a href="development-documents/library-packaging.html"><strong aria-hidden="true">3.6.</strong> Library Packaging</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="development-documents/library-packaging/apple.html"><strong aria-hidden="true">3.6.1.</strong> Apple</a></li><li class="chapter-item expanded "><a href="development-documents/library-packaging/android.html"><strong aria-hidden="true">3.6.2.</strong> Android</a></li><li class="chapter-item expanded "><a href="development-documents/library-packaging/web.html"><strong aria-hidden="true">3.6.3.</strong> Web</a></li></ol></li></ol></li><li class="chapter-item expanded "><a href="appendix/index.html"><strong aria-hidden="true">4.</strong> Appendix</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="appendix/link-collection.html"><strong aria-hidden="true">4.1.</strong> Link Collection</a></li></ol></li><li class="chapter-item expanded "><a href="rfc/0001-rfc-process.html"><strong aria-hidden="true">5.</strong> RFCs</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="rfc/0000-template.html"><strong aria-hidden="true">5.1.</strong> 0000-template</a></li><li class="chapter-item expanded "><a href="rfc/0001-rfc-process.html"><strong aria-hidden="true">5.2.</strong> 0001-rfc-process</a></li></ol></li></ol>';
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
