/* ea:   0x0006A148 */
/* name: doexit */
/* size: 212 */
int (*__fastcall doexit(int a1, int a2, int a3))(void)
{
  void (**v5)(void); // r1
  void (**i)(void); // r4
  int (*result)(void); // r0

  byte_81235 = a3;
  if ( a2 == 0 )
  {
    v5 = (void (**)(void))dword_8123C;
    if ( dword_8123C != 0 )
    {
      for ( i = (void (**)(void))(dword_81238 - 4); i >= v5; --i )
      {
        if ( *i != nullptr )
        {
          (*i)();
          v5 = (void (**)(void))dword_8123C;
        }
      }
      LocalFree(v5);
      dword_81238 = 0;
      dword_8123C = 0;
    }
    initterm((int (*)(void))&unk_7D1F8, (unsigned int)&unk_7D1FC);
  }
  result = initterm((int (*)(void))&unk_7D200, (unsigned int)&unk_7D204);
  if ( a3 == 0 )
    return (int (*)(void))MEMORY[0xF000F7F8](66, a1);
  return result;
}

