use tauri::AppHandle;

#[tauri::command]
pub fn seed_knr_book(app: AppHandle) -> Result<Vec<String>, String> {
    let pdir = crate::projects_dir(&app);
    let mut created: Vec<String> = Vec::new();

    const ATTRIBUTION_MD: &str = "\
# Source: The C Programming Language

These playground examples are based on the curriculum of:

**\"The C Programming Language\" (2nd Edition)**
by Brian W. Kernighan and Dennis M. Ritchie.

| | |
|---|---|
| Known as       | K&R C |
| Authors        | Brian W. Kernighan, Dennis M. Ritchie |
| Edition        | 2nd (ANSI C), 1988 |
| Publisher      | Prentice Hall |

## Notes

The playground code is original educational C written to illustrate the
same concepts covered in each chapter. It is not verbatim source from the
book. Rustic Playground is not affiliated with or endorsed by the authors
or publisher.
";

    // (project_name, [(playground_name, code)])
    #[allow(clippy::type_complexity)]
    let chapters: Vec<(&str, Vec<(&str, &str)>)> = vec![
        // ── Chapter 1: A Tutorial Introduction ───────────────────────────────
        (
            "knr_ch01_tutorial",
            vec![
                (
                    "hello.c",
                    r#"// K&R Ch 1 – Hello, World!
// The first C program. printf is from <stdio.h>.

#include <stdio.h>

int main() {
    printf("Hello, world!\n");

    // Format specifiers: %d int, %f float, %s string, %c char
    int year = 1972;
    printf("C was created in %d.\n", year);
    printf("%d + %d = %d\n", 1, 2, 1 + 2);

    // Width and precision
    printf("Pi is approximately %.2f\n", 3.14159);
    printf("%10s | %10s\n", "Item", "Price");
    printf("%10s | %10.2f\n", "Coffee", 4.50);
    printf("%10s | %10.2f\n", "Muffin", 3.25);

    return 0;
}
"#,
                ),
                (
                    "fahrenheit.c",
                    r#"// K&R Ch 1 – Temperature Conversion
// The classic Fahrenheit-to-Celsius table from K&R.

#include <stdio.h>

int main() {
    // Integer version (K&R style)
    printf("Fahrenheit-Celsius (integer)\n");
    printf("%6s %6s\n", "F", "C");
    printf("------+------\n");
    for (int fahr = 0; fahr <= 300; fahr += 20) {
        int celsius = 5 * (fahr - 32) / 9;
        printf("%6d %6d\n", fahr, celsius);
    }

    printf("\n");

    // Floating-point version (more accurate)
    printf("Fahrenheit-Celsius (float)\n");
    printf("%6s %6s\n", "F", "C");
    printf("------+------\n");
    for (double fahr = 0.0; fahr <= 300.0; fahr += 20.0) {
        double celsius = (5.0 / 9.0) * (fahr - 32.0);
        printf("%6.1f %6.1f\n", fahr, celsius);
    }

    return 0;
}
"#,
                ),
                (
                    "char_counting.c",
                    r#"// K&R Ch 1 – Character, Word, and Line Counting
// Demonstrates getchar() loop, the fundamental C I/O pattern.

#include <stdio.h>

int main() {
    // We'll count characters, words, and lines in a sample string
    // (K&R reads from stdin; here we use a string for the playground)
    const char *text =
        "The quick brown fox\n"
        "jumps over the lazy dog.\n"
        "Pack my box with five dozen liquor jugs.\n";

    long chars = 0, words = 0, lines = 0;
    int in_word = 0;

    for (int i = 0; text[i] != '\0'; i++) {
        char c = text[i];
        chars++;

        if (c == '\n') {
            lines++;
        }

        if (c == ' ' || c == '\n' || c == '\t') {
            in_word = 0;
        } else if (!in_word) {
            in_word = 1;
            words++;
        }
    }

    printf("Text:\n%s\n", text);
    printf("Characters: %ld\n", chars);
    printf("Words:      %ld\n", words);
    printf("Lines:      %ld\n", lines);

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 2: Types, Operators, and Expressions ─────────────────────
        (
            "knr_ch02_types",
            vec![
                (
                    "data_types.c",
                    r#"// K&R Ch 2 – Data Types and Sizes
// C's fundamental types: char, int, float, double, and their sizes.

#include <stdio.h>
#include <limits.h>
#include <float.h>

int main() {
    // Basic types and their sizes
    printf("%-12s  %2zu bytes\n", "char",      sizeof(char));
    printf("%-12s  %2zu bytes\n", "short",     sizeof(short));
    printf("%-12s  %2zu bytes\n", "int",       sizeof(int));
    printf("%-12s  %2zu bytes\n", "long",      sizeof(long));
    printf("%-12s  %2zu bytes\n", "long long", sizeof(long long));
    printf("%-12s  %2zu bytes\n", "float",     sizeof(float));
    printf("%-12s  %2zu bytes\n", "double",    sizeof(double));
    printf("%-12s  %2zu bytes\n", "pointer",   sizeof(void *));

    printf("\nInteger ranges:\n");
    printf("  char:      %d to %d\n", CHAR_MIN, CHAR_MAX);
    printf("  int:       %d to %d\n", INT_MIN, INT_MAX);
    printf("  long:      %ld to %ld\n", LONG_MIN, LONG_MAX);

    printf("\nFloat precision:\n");
    printf("  float:  %d significant digits\n", FLT_DIG);
    printf("  double: %d significant digits\n", DBL_DIG);

    // Type conversions
    int i = 42;
    float f = i;           // implicit: int -> float
    int truncated = 3.99;  // implicit: float -> int (truncates)
    printf("\nConversions:\n");
    printf("  int %d -> float %f\n", i, f);
    printf("  3.99 -> int %d (truncated)\n", truncated);

    return 0;
}
"#,
                ),
                (
                    "operators.c",
                    r#"// K&R Ch 2 – Operators and Expressions
// Arithmetic, relational, logical, and bitwise operators.

#include <stdio.h>

int main() {
    // Arithmetic
    int a = 17, b = 5;
    printf("a = %d, b = %d\n", a, b);
    printf("  a + b  = %d\n", a + b);
    printf("  a - b  = %d\n", a - b);
    printf("  a * b  = %d\n", a * b);
    printf("  a / b  = %d  (integer division)\n", a / b);
    printf("  a %% b  = %d  (remainder)\n", a % b);

    // Relational and logical
    printf("\nRelational & logical:\n");
    printf("  a > b  = %d\n", a > b);
    printf("  a == b = %d\n", a == b);
    printf("  !0 = %d, !1 = %d\n", !0, !1);
    printf("  1 && 0 = %d\n", 1 && 0);
    printf("  1 || 0 = %d\n", 1 || 0);

    // Bitwise operators
    unsigned int x = 0xAB;  // 10101011
    unsigned int y = 0xCD;  // 11001101
    printf("\nBitwise (hex):\n");
    printf("  x      = 0x%02X\n", x);
    printf("  y      = 0x%02X\n", y);
    printf("  x & y  = 0x%02X  (AND)\n", x & y);
    printf("  x | y  = 0x%02X  (OR)\n", x | y);
    printf("  x ^ y  = 0x%02X  (XOR)\n", x ^ y);
    printf("  ~x     = 0x%02X  (NOT, lower 8 bits)\n", (~x) & 0xFF);
    printf("  x << 2 = 0x%02X  (left shift)\n", (x << 2) & 0xFFFF);
    printf("  x >> 2 = 0x%02X  (right shift)\n", x >> 2);

    // Increment/decrement and conditional
    int n = 5;
    printf("\nIncrement: n=%d, n++=%d, now n=%d\n", 5, n++, n);
    printf("Conditional: %s\n", (a > b) ? "a is greater" : "b is greater");

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 3: Control Flow ──────────────────────────────────────────
        (
            "knr_ch03_control_flow",
            vec![
                (
                    "if_else.c",
                    r#"// K&R Ch 3 – If-Else and Switch
// Branching: if-else chains and switch statements.

#include <stdio.h>

// Binary search — classic K&R example
int binsearch(int x, int v[], int n) {
    int low = 0, high = n - 1;
    while (low <= high) {
        int mid = (low + high) / 2;
        if (x < v[mid])
            high = mid - 1;
        else if (x > v[mid])
            low = mid + 1;
        else
            return mid;  // found
    }
    return -1;  // not found
}

int main() {
    int arr[] = {2, 5, 8, 12, 16, 23, 38, 42, 56, 72, 91};
    int n = sizeof(arr) / sizeof(arr[0]);

    int targets[] = {23, 42, 7, 91, 1};
    int ntargets = sizeof(targets) / sizeof(targets[0]);

    printf("Binary search:\n");
    for (int i = 0; i < ntargets; i++) {
        int idx = binsearch(targets[i], arr, n);
        if (idx >= 0)
            printf("  %2d found at index %d\n", targets[i], idx);
        else
            printf("  %2d not found\n", targets[i]);
    }

    // Switch statement — count digit types
    const char *input = "abc 123 !@# 456 xyz";
    int digits = 0, letters = 0, spaces = 0, other = 0;

    for (int i = 0; input[i]; i++) {
        char c = input[i];
        if (c >= '0' && c <= '9')      digits++;
        else if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) letters++;
        else if (c == ' ')             spaces++;
        else                           other++;
    }

    printf("\nCharacter classification of \"%s\":\n", input);
    printf("  digits:  %d\n", digits);
    printf("  letters: %d\n", letters);
    printf("  spaces:  %d\n", spaces);
    printf("  other:   %d\n", other);

    return 0;
}
"#,
                ),
                (
                    "loops.c",
                    r#"// K&R Ch 3 – Loops: while, for, do-while
// The three loop forms and break/continue.

#include <stdio.h>
#include <string.h>

// Reverse a string in place (K&R classic)
void reverse(char s[]) {
    int i, j;
    for (i = 0, j = strlen(s) - 1; i < j; i++, j--) {
        char temp = s[i];
        s[i] = s[j];
        s[j] = temp;
    }
}

// Convert integer to string (itoa from K&R)
void itoa(int n, char s[]) {
    int sign = n;
    if (sign < 0) n = -n;

    int i = 0;
    do {
        s[i++] = n % 10 + '0';
    } while ((n /= 10) > 0);

    if (sign < 0) s[i++] = '-';
    s[i] = '\0';
    reverse(s);
}

int main() {
    // While loop — Collatz sequence
    printf("Collatz sequence from 27:\n  ");
    int n = 27;
    int steps = 0;
    while (n != 1) {
        printf("%d -> ", n);
        n = (n % 2 == 0) ? n / 2 : 3 * n + 1;
        steps++;
        if (steps % 10 == 0) printf("\n  ");
    }
    printf("1 (%d steps)\n\n", steps);

    // For loop with reverse
    char str[] = "Hello, K&R!";
    printf("Original: \"%s\"\n", str);
    reverse(str);
    printf("Reversed: \"%s\"\n\n", str);

    // Do-while with itoa
    int values[] = {0, 42, -123, 9999};
    for (int i = 0; i < 4; i++) {
        char buf[32];
        itoa(values[i], buf);
        printf("itoa(%d) = \"%s\"\n", values[i], buf);
    }

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 4: Functions and Program Structure ───────────────────────
        (
            "knr_ch04_functions",
            vec![
                (
                    "functions.c",
                    r#"// K&R Ch 4 – Functions
// Function declarations, definitions, and scope.

#include <stdio.h>
#include <ctype.h>

// Function declaration (prototype)
double power(double base, int exp);
int atoi_simple(const char s[]);

// K&R's power function
double power(double base, int exp) {
    double result = 1.0;
    for (int i = 0; i < exp; i++)
        result *= base;
    return result;
}

// K&R's atoi — convert string to integer
int atoi_simple(const char s[]) {
    int i = 0, n = 0, sign = 1;

    // Skip whitespace
    while (isspace(s[i])) i++;

    // Optional sign
    if (s[i] == '+' || s[i] == '-')
        sign = (s[i++] == '-') ? -1 : 1;

    // Convert digits
    while (isdigit(s[i]))
        n = 10 * n + (s[i++] - '0');

    return sign * n;
}

int main() {
    // Power function
    printf("Powers of 2:\n");
    for (int i = 0; i <= 10; i++)
        printf("  2^%2d = %7.0f\n", i, power(2.0, i));

    // atoi
    const char *strings[] = {"  42", "-123", "+99", "0", "  -0007"};
    printf("\natoi conversions:\n");
    for (int i = 0; i < 5; i++)
        printf("  \"%s\" -> %d\n", strings[i], atoi_simple(strings[i]));

    return 0;
}
"#,
                ),
                (
                    "recursion.c",
                    r#"// K&R Ch 4 – Recursion
// Recursive functions: factorial, Fibonacci, quicksort.

#include <stdio.h>

long factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

long fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// K&R quicksort
void swap(int v[], int i, int j) {
    int temp = v[i];
    v[i] = v[j];
    v[j] = temp;
}

void qsort_kr(int v[], int left, int right) {
    if (left >= right) return;

    swap(v, left, (left + right) / 2);
    int last = left;
    for (int i = left + 1; i <= right; i++) {
        if (v[i] < v[left])
            swap(v, ++last, i);
    }
    swap(v, left, last);
    qsort_kr(v, left, last - 1);
    qsort_kr(v, last + 1, right);
}

int main() {
    // Factorial
    printf("Factorials:\n");
    for (int i = 0; i <= 12; i++)
        printf("  %2d! = %ld\n", i, factorial(i));

    // Fibonacci
    printf("\nFibonacci sequence:\n  ");
    for (int i = 0; i < 15; i++)
        printf("%ld ", fibonacci(i));
    printf("\n");

    // Quicksort
    int arr[] = {64, 25, 12, 22, 11, 90, 33, 7, 55, 41};
    int n = sizeof(arr) / sizeof(arr[0]);

    printf("\nBefore sort: ");
    for (int i = 0; i < n; i++) printf("%d ", arr[i]);

    qsort_kr(arr, 0, n - 1);

    printf("\nAfter sort:  ");
    for (int i = 0; i < n; i++) printf("%d ", arr[i]);
    printf("\n");

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 5: Pointers and Arrays ───────────────────────────────────
        (
            "knr_ch05_pointers",
            vec![
                (
                    "pointers.c",
                    r#"// K&R Ch 5 – Pointers and Addresses
// Pointer basics: &, *, pointer arithmetic, arrays.

#include <stdio.h>

// Swap using pointers (K&R classic)
void swap(int *px, int *py) {
    int temp = *px;
    *px = *py;
    *py = temp;
}

// strlen using pointer arithmetic
int str_len(const char *s) {
    const char *p = s;
    while (*p != '\0') p++;
    return p - s;
}

// strcpy using pointers (K&R style)
void str_copy(char *dest, const char *src) {
    while ((*dest++ = *src++) != '\0')
        ;
}

int main() {
    // Pointer basics
    int x = 1, y = 2;
    printf("Before swap: x=%d, y=%d\n", x, y);
    swap(&x, &y);
    printf("After swap:  x=%d, y=%d\n", x, y);

    // Pointer arithmetic with arrays
    int arr[] = {10, 20, 30, 40, 50};
    int *p = arr;  // points to arr[0]

    printf("\nArray via pointer arithmetic:\n");
    for (int i = 0; i < 5; i++)
        printf("  *(p+%d) = %d  (same as arr[%d] = %d)\n",
               i, *(p + i), i, arr[i]);

    // Pointer difference
    int *first = &arr[0];
    int *last  = &arr[4];
    printf("\nPointer difference: last - first = %ld elements\n", last - first);

    // String functions with pointers
    const char *hello = "Hello, K&R!";
    printf("\nstr_len(\"%s\") = %d\n", hello, str_len(hello));

    char buf[64];
    str_copy(buf, hello);
    printf("str_copy result: \"%s\"\n", buf);

    return 0;
}
"#,
                ),
                (
                    "ptr_arrays.c",
                    r#"// K&R Ch 5 – Pointer Arrays and Multi-dimensional Arrays
// Arrays of pointers, command-line style argument handling.

#include <stdio.h>
#include <string.h>

// Sort an array of strings using pointers (K&R-style)
void sort_strings(char *lines[], int nlines) {
    for (int i = 0; i < nlines - 1; i++) {
        for (int j = i + 1; j < nlines; j++) {
            if (strcmp(lines[i], lines[j]) > 0) {
                char *temp = lines[i];
                lines[i] = lines[j];
                lines[j] = temp;
            }
        }
    }
}

// 2D array: matrix operations
void print_matrix(int rows, int cols, int mat[rows][cols]) {
    for (int i = 0; i < rows; i++) {
        printf("  [ ");
        for (int j = 0; j < cols; j++)
            printf("%3d ", mat[i][j]);
        printf("]\n");
    }
}

int main() {
    // Array of string pointers
    char *months[] = {
        "January", "February", "March", "April",
        "May", "June", "July", "August",
        "September", "October", "November", "December"
    };
    int nmonths = 12;

    printf("Months (sorted alphabetically):\n");
    sort_strings(months, nmonths);
    for (int i = 0; i < nmonths; i++)
        printf("  %2d. %s\n", i + 1, months[i]);

    // 2D array
    int matrix[3][4] = {
        { 1,  2,  3,  4},
        { 5,  6,  7,  8},
        { 9, 10, 11, 12}
    };

    printf("\n3x4 Matrix:\n");
    print_matrix(3, 4, matrix);

    // Row and column sums
    printf("\nRow sums: ");
    for (int i = 0; i < 3; i++) {
        int sum = 0;
        for (int j = 0; j < 4; j++) sum += matrix[i][j];
        printf("%d ", sum);
    }
    printf("\nCol sums: ");
    for (int j = 0; j < 4; j++) {
        int sum = 0;
        for (int i = 0; i < 3; i++) sum += matrix[i][j];
        printf("%d ", sum);
    }
    printf("\n");

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 6: Structures ────────────────────────────────────────────
        (
            "knr_ch06_structures",
            vec![
                (
                    "structs.c",
                    r#"// K&R Ch 6 – Structures
// Defining and using structs, nested structs, arrays of structs.

#include <stdio.h>
#include <string.h>
#include <math.h>

// K&R point and rectangle
struct point {
    double x;
    double y;
};

struct rect {
    struct point pt1;  // lower-left
    struct point pt2;  // upper-right
};

struct point make_point(double x, double y) {
    struct point p;
    p.x = x;
    p.y = y;
    return p;
}

double distance(struct point a, struct point b) {
    double dx = a.x - b.x;
    double dy = a.y - b.y;
    return sqrt(dx * dx + dy * dy);
}

double rect_area(struct rect r) {
    return fabs((r.pt2.x - r.pt1.x) * (r.pt2.y - r.pt1.y));
}

int pt_in_rect(struct point p, struct rect r) {
    return p.x >= r.pt1.x && p.x <= r.pt2.x &&
           p.y >= r.pt1.y && p.y <= r.pt2.y;
}

// K&R keyword counting (simplified)
struct key {
    const char *word;
    int count;
};

int main() {
    // Points and rectangles
    struct point a = make_point(0.0, 0.0);
    struct point b = make_point(3.0, 4.0);
    printf("Distance from (%.1f,%.1f) to (%.1f,%.1f): %.2f\n",
           a.x, a.y, b.x, b.y, distance(a, b));

    struct rect screen = { {0, 0}, {1920, 1080} };
    printf("Screen area: %.0f pixels\n", rect_area(screen));

    struct point cursor = {500, 300};
    printf("(%g,%g) in screen? %s\n",
           cursor.x, cursor.y,
           pt_in_rect(cursor, screen) ? "yes" : "no");

    // Keyword counting
    struct key keywords[] = {
        {"if", 0}, {"else", 0}, {"for", 0},
        {"while", 0}, {"return", 0}, {"int", 0}
    };
    int nkeys = sizeof(keywords) / sizeof(keywords[0]);

    const char *code[] = {
        "int", "for", "if", "return", "if", "else",
        "while", "for", "int", "return", "for", "if"
    };
    int nwords = sizeof(code) / sizeof(code[0]);

    for (int i = 0; i < nwords; i++) {
        for (int j = 0; j < nkeys; j++) {
            if (strcmp(code[i], keywords[j].word) == 0) {
                keywords[j].count++;
                break;
            }
        }
    }

    printf("\nKeyword frequencies:\n");
    for (int i = 0; i < nkeys; i++) {
        if (keywords[i].count > 0)
            printf("  %-8s %d\n", keywords[i].word, keywords[i].count);
    }

    return 0;
}
"#,
                ),
                (
                    "self_ref.c",
                    r#"// K&R Ch 6 – Self-Referential Structures
// Binary search tree — K&R's word frequency counter pattern.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct tnode {
    char *word;
    int count;
    struct tnode *left;
    struct tnode *right;
};

struct tnode *talloc(void) {
    return (struct tnode *)malloc(sizeof(struct tnode));
}

char *str_dup(const char *s) {
    char *p = malloc(strlen(s) + 1);
    if (p) strcpy(p, s);
    return p;
}

// Add a word to the tree (or increment its count)
struct tnode *addtree(struct tnode *p, const char *w) {
    if (p == NULL) {
        p = talloc();
        p->word = str_dup(w);
        p->count = 1;
        p->left = p->right = NULL;
    } else if (strcmp(w, p->word) < 0) {
        p->left = addtree(p->left, w);
    } else if (strcmp(w, p->word) > 0) {
        p->right = addtree(p->right, w);
    } else {
        p->count++;
    }
    return p;
}

// In-order traversal: prints words alphabetically
void treeprint(struct tnode *p) {
    if (p != NULL) {
        treeprint(p->left);
        printf("  %-12s %d\n", p->word, p->count);
        treeprint(p->right);
    }
}

void treefree(struct tnode *p) {
    if (p != NULL) {
        treefree(p->left);
        treefree(p->right);
        free(p->word);
        free(p);
    }
}

int main() {
    const char *words[] = {
        "the", "quick", "brown", "fox", "jumps", "over",
        "the", "lazy", "dog", "the", "fox", "the",
        "brown", "quick", "fox", "over", "the"
    };
    int nwords = sizeof(words) / sizeof(words[0]);

    struct tnode *root = NULL;
    for (int i = 0; i < nwords; i++)
        root = addtree(root, words[i]);

    printf("Word frequencies (alphabetical):\n");
    treeprint(root);
    treefree(root);

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 7: Input and Output ──────────────────────────────────────
        (
            "knr_ch07_io",
            vec![
                (
                    "formatted_io.c",
                    r#"// K&R Ch 7 – Formatted I/O
// printf/sprintf formatting, sscanf parsing.

#include <stdio.h>
#include <string.h>

int main() {
    // printf format specifiers
    printf("=== Printf Format Gallery ===\n\n");

    // Integers
    printf("Integers:\n");
    printf("  decimal:  %d\n", 255);
    printf("  octal:    %o\n", 255);
    printf("  hex:      %x (or %X)\n", 255, 255);
    printf("  padded:   [%10d]\n", 42);
    printf("  left:     [%-10d]\n", 42);
    printf("  zero-pad: [%010d]\n", 42);

    printf("\nFloats:\n");
    double pi = 3.14159265358979;
    printf("  default:    %f\n", pi);
    printf("  precision:  %.2f\n", pi);
    printf("  scientific: %e\n", pi);
    printf("  shortest:   %g\n", pi);
    printf("  width:      [%12.4f]\n", pi);

    printf("\nStrings:\n");
    printf("  plain:    [%s]\n", "hello");
    printf("  width:    [%20s]\n", "hello");
    printf("  left:     [%-20s]\n", "hello");
    printf("  truncate: [%.3s]\n", "hello");

    // sprintf — format into a buffer
    char buf[128];
    sprintf(buf, "User: %s, Age: %d, Score: %.1f%%", "Alice", 30, 95.5);
    printf("\nsprintf result: \"%s\"\n", buf);

    // sscanf — parse from a string
    const char *data = "Bob 25 87.3";
    char name[32];
    int age;
    double score;
    sscanf(data, "%s %d %lf", name, &age, &score);
    printf("\nsscanf parsed: name=%s, age=%d, score=%.1f\n", name, age, score);

    return 0;
}
"#,
                ),
                (
                    "file_io.c",
                    r#"// K&R Ch 7 – File I/O
// fopen, fclose, fgets, fprintf — reading and writing files.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    // Get the content folder path
    const char *content_dir = getenv("PLAYGROUND_CONTENT");
    if (!content_dir) content_dir = ".";

    char path[256];

    // Write a data file
    snprintf(path, sizeof(path), "%s/data.txt", content_dir);
    FILE *fp = fopen(path, "w");
    if (!fp) { perror("fopen"); return 1; }

    fprintf(fp, "Name       Score  Grade\n");
    fprintf(fp, "Alice       95     A\n");
    fprintf(fp, "Bob         87     B\n");
    fprintf(fp, "Charlie     92     A\n");
    fprintf(fp, "Diana       78     C\n");
    fprintf(fp, "Eve         88     B\n");
    fclose(fp);
    printf("Wrote %s\n\n", path);

    // Read it back
    fp = fopen(path, "r");
    if (!fp) { perror("fopen"); return 1; }

    char line[128];
    int lineno = 0;
    int total = 0, count = 0;

    printf("Contents:\n");
    while (fgets(line, sizeof(line), fp)) {
        lineno++;
        printf("  %2d: %s", lineno, line);

        // Parse score from data lines (skip header)
        if (lineno > 1) {
            char name[32], grade[4];
            int score;
            if (sscanf(line, "%s %d %s", name, &score, grade) == 3) {
                total += score;
                count++;
            }
        }
    }
    fclose(fp);

    if (count > 0) {
        printf("\n  Average score: %.1f\n", (double)total / count);
    }

    return 0;
}
"#,
                ),
            ],
        ),
        // ── Chapter 8: The UNIX System Interface ─────────────────────────────
        (
            "knr_ch08_system",
            vec![
                (
                    "memory.c",
                    r#"// K&R Ch 8 – Dynamic Memory
// malloc, free, realloc — building a dynamic array.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// A growable array of ints
typedef struct {
    int *data;
    int len;
    int cap;
} IntVec;

IntVec vec_new(void) {
    IntVec v;
    v.cap = 4;
    v.len = 0;
    v.data = malloc(v.cap * sizeof(int));
    return v;
}

void vec_push(IntVec *v, int value) {
    if (v->len == v->cap) {
        v->cap *= 2;
        v->data = realloc(v->data, v->cap * sizeof(int));
    }
    v->data[v->len++] = value;
}

void vec_print(const IntVec *v) {
    printf("[");
    for (int i = 0; i < v->len; i++) {
        if (i > 0) printf(", ");
        printf("%d", v->data[i]);
    }
    printf("] (len=%d, cap=%d)\n", v->len, v->cap);
}

void vec_free(IntVec *v) {
    free(v->data);
    v->data = NULL;
    v->len = v->cap = 0;
}

int main() {
    printf("=== Dynamic Array (IntVec) ===\n\n");

    IntVec v = vec_new();
    printf("Empty:  "); vec_print(&v);

    for (int i = 1; i <= 10; i++) {
        vec_push(&v, i * 10);
        printf("Push %3d: ", i * 10); vec_print(&v);
    }

    // Manual memory: string duplication
    printf("\n=== String Duplication ===\n\n");
    const char *original = "Hello, K&R!";
    size_t slen = strlen(original);
    char *copy = malloc(slen + 1);
    memcpy(copy, original, slen + 1);

    printf("Original: \"%s\" at %p\n", original, (void *)original);
    printf("Copy:     \"%s\" at %p\n", copy, (void *)copy);
    printf("Same pointer? %s\n", (original == copy) ? "yes" : "no");

    free(copy);
    vec_free(&v);

    printf("\nAll memory freed.\n");
    return 0;
}
"#,
                ),
                (
                    "sqlite.c",
                    r#"// K&R Ch 8 – System Libraries: SQLite
// Using sqlite3 (ships with macOS) for structured data.

#include <stdio.h>
#include <sqlite3.h>

static int print_row(void *label, int ncols, char **values, char **names) {
    const char *prefix = label ? (const char *)label : "";
    printf("%s", prefix);
    for (int i = 0; i < ncols; i++) {
        printf("%s=%s", names[i], values[i] ? values[i] : "NULL");
        if (i < ncols - 1) printf(", ");
    }
    printf("\n");
    return 0;
}

int main() {
    sqlite3 *db;
    char *err = NULL;

    if (sqlite3_open(":memory:", &db) != SQLITE_OK) {
        fprintf(stderr, "Cannot open database: %s\n", sqlite3_errmsg(db));
        return 1;
    }
    printf("SQLite version: %s\n\n", sqlite3_libversion());

    // Create and populate a table
    const char *setup =
        "CREATE TABLE books ("
        "  id INTEGER PRIMARY KEY,"
        "  title TEXT NOT NULL,"
        "  author TEXT NOT NULL,"
        "  year INTEGER"
        ");"
        "INSERT INTO books (title, author, year) VALUES"
        "  ('The C Programming Language', 'Kernighan & Ritchie', 1978),"
        "  ('Structure and Interpretation', 'Abelson & Sussman', 1985),"
        "  ('The Art of Computer Programming', 'Knuth', 1968),"
        "  ('Programming Pearls', 'Bentley', 1986),"
        "  ('The Mythical Man-Month', 'Brooks', 1975);";

    if (sqlite3_exec(db, setup, NULL, NULL, &err) != SQLITE_OK) {
        fprintf(stderr, "SQL error: %s\n", err);
        sqlite3_free(err);
        sqlite3_close(db);
        return 1;
    }

    printf("All books (by year):\n");
    sqlite3_exec(db, "SELECT * FROM books ORDER BY year;",
                 print_row, "  ", NULL);

    printf("\nBooks before 1980:\n");
    sqlite3_exec(db,
        "SELECT title, year FROM books WHERE year < 1980 ORDER BY year;",
        print_row, "  ", NULL);

    printf("\nCount and average year:\n");
    sqlite3_exec(db,
        "SELECT COUNT(*) as count, ROUND(AVG(year)) as avg_year FROM books;",
        print_row, "  ", NULL);

    sqlite3_close(db);
    return 0;
}
"#,
                ),
            ],
        ),
    ]; // end of chapters vec

    for (chapter, playgrounds) in &chapters {
        let project_path = pdir.join(chapter);
        if project_path.exists() {
            continue;
        } // idempotent — skip if already there

        std::fs::create_dir_all(&project_path)
            .map_err(|e| format!("seed {chapter}: {e}"))?;

        // Write native manifest
        let manifest = crate::rustic_manifest::new_native_manifest();
        crate::rustic_manifest::write_manifest(&project_path, &manifest)?;

        // Content directory + attribution
        let content_dir = project_path.join("content");
        std::fs::create_dir_all(&content_dir)
            .map_err(|e| format!("seed {chapter} content/: {e}"))?;
        std::fs::write(content_dir.join("attribution.md"), ATTRIBUTION_MD)
            .map_err(|e| format!("seed {chapter} attribution.md: {e}"))?;

        for (pg, code) in playgrounds {
            std::fs::write(project_path.join(pg), code)
                .map_err(|e| format!("seed {chapter}/{pg}: {e}"))?;
        }
        created.push(chapter.to_string());
    }

    Ok(created)
}
