@EXTERNAL FUNCTION puts : DINT
VAR_INPUT
    text : STRING;
END_VAR
END_FUNCTION

@EXTERNAL FUNCTION exit : DINT
VAR_INPUT
    status : DINT;
END_VAR
END_FUNCTION

FUNCTION _start : DINT
	puts('hello, world!');
    exit(0);
END_FUNCTION
