# TODO

- [x] Better error handling. If an unhandled error occurs then the entire application should exit instead of making the output dirty
    This is mainly for in async requests. If an error occurs it should be reported instead of throwing
    - [ ] this needs more work for now it logs to a file in app %local%/rataify/errors.log
- [ ] Better way of seeing which part of the landing page is being interacted with
- [x] Way of having actions update parts of state depending on context
    - [ ] Implement callback on actions for save and remove all actions
- [ ] Make individual modal and window states handle key input individually. Ex. up, down, left, right, tab, backtab 
- [ ] Scrollable landing page descriptions
- [ ] Add entire context to queue

## Done

- [x] Ensure that every spotify api call is wrapped in a token refresh if needed
- [x] Way of interacting with subject of landing page (playlist, album, etc.)
- [x] Custom landing pages for each type
- [x] Wrapping tabs in library
- [x] Parse descriptions for italic, bold, and line break html
- [x] Playing from landing page plays from context with offset
