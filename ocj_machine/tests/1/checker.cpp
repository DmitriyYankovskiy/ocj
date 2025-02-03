#include<bits/stdc++.h>
using namespace std;
int main(int argc, char** argv) {
    string ideal, res;
    ideal = argv[1]; 
    res = argv[2];

    ifstream in_ideal(ideal);
    ifstream in_res(res);

    int a, b;
    in_ideal >> a;
    in_res >> b;

    if (a == b) {
        cout << "Ok";
    } else {
        cout << "Wa";
    }
}