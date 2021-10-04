// This optionnal mkdocs add-on eases navigation when the sidenav
// is big by hiding items of the not selected main items.
;(function(main){
	let sidenav = document.querySelector("ul.nav.bs-sidenav");
	let hasScrollbar = sidenav.scrollHeight > sidenav.clientHeight;
	if (!hasScrollbar) return; // the standard behavior is fine
	let items = document.querySelectorAll(".nav-item");
	function open(mainItem) {
		mainItem.classList.add("at-selected");
		for (
			let item = mainItem.nextElementSibling;
			item && !item.classList.contains("main");
			item = item.nextElementSibling
		) {
			item.style.display = "block";
		}
	}
	function closeAll(){
		for (let item of items) {
			if (item.classList.contains("main")) {
				item.classList.remove("at-selected");
			} else {
				item.style.display = "none";
			}
		}
	}
	function onMainClicked(){
		let opening = !this.classList.contains("at-selected");
		closeAll();
		if (opening) open(this);
	}
	function extractUrlHash(url){
		let match = url.match(/#[^?&]+/);
		return match ? match[0] : null;
	}
	function extractItemHash(item){
		let a = item.querySelector("a");
		return a ? extractUrlHash(a.href) : null;
	}
	function showHash(hash) {
		let lastMain;
		for (let item of items) {
			if (item.classList.contains("main")) {
				lastMain = item;
			}
			let itemHash = extractItemHash(item);
			if (itemHash && itemHash == hash) {
				open(lastMain);
				return;
			}
		}
	}

	// adding a listener on all nav-item elements to make them
	// open or close the sub-items
	;[].forEach.call(items, item => {
		if (item.classList.contains("main")) {
			item.addEventListener("click", onMainClicked);
			lastMain = item;
		} else {
			item.style.display = "none";
		}
	});
	// if we came to the page with a hash, we open the relevant part of
	// the nav
	if (document.location.hash) {
		showHash(document.location.hash);
	}
	// hook on internal links
	for (let a of document.querySelectorAll(".col-md-9 a")) {
		a.addEventListener("click", function(){
			closeAll();
			showHash(extractUrlHash(a.href));
		});
	}
	let lastActive = sidenav.querySelector(".active");
	document.addEventListener("scroll", function(){
		let active = sidenav.querySelector(".active");
		if (active != lastActive) {
			lastActive = active;
			let activeHash = extractItemHash(active);
			closeAll();
			showHash(activeHash);
		}
	});
})();
