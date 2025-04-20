A structure that holds:
- pointer to og fn
- pre-fn hooks
    - passed in parameters and a fn for the next step
    - Returns an option with a tuple of the same type
- post-fn hooks