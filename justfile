test:
	(find programs && find tests) | entr -cs 'anchor test'
