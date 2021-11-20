enum struct FooEnum
{
	char name[MAX_NAME_LENGTH];
	int id;
	char fullAccountID[32];

	void Init(int foo){
		return;
	}

	int Test(const char[] foo){
		return 1;
	}
}