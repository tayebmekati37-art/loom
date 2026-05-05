       IDENTIFICATION DIVISION.
       PROGRAM-ID. TESTIO.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT INFILE ASSIGN TO 'input.txt'.
       DATA DIVISION.
       FILE SECTION.
       FD INFILE.
       01 IN-RECORD PIC X(80).
       WORKING-STORAGE SECTION.
       01 WS-X PIC 9(3) VALUE 5.
       01 WS-Y PIC 9(3) VALUE 0.
       PROCEDURE DIVISION.
           OPEN INPUT INFILE
           READ INFILE INTO IN-RECORD
           PERFORM PARA-A
           IF WS-X > 10 THEN
               MOVE 1 TO WS-Y
           ELSE
               MOVE 0 TO WS-Y
           END-IF
           DISPLAY WS-Y
           CLOSE INFILE
           STOP RUN.
       PARA-A.
           ADD 10 TO WS-X.