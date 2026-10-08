/* ea:   0x00064CFC */
/* name: sub_64CFC */
/* size: 152 */
char *__fastcall sub_64CFC(int a1, int a2, unsigned int a3, char *a4, int a5, int a6)
{
  char *v7; // r8
  unsigned int v8; // r5
  char *v9; // r4
  char v10; // r3
  char v11; // t1
  char v12; // r1
  char v13; // r3
  char v14; // t1
  int i; // r1

  v7 = a4;
  v8 = 0;
  if ( a3 == 0 )
    return v7;
  v9 = &a4[-a5];
  while ( (int)v9 < a6 )
  {
    v11 = *v7++;
    v10 = v11;
    ++v9;
    v12 = v11;
    if ( (v11 & 0xC0) == 0xC0 )
    {
      v14 = *v7++;
      v13 = v14;
      ++v9;
      for ( i = v12 & 0x3F; i != 0; ++v8 )
      {
        *(_BYTE *)(v8 + a2) = v13;
        i = (unsigned __int8)(i - 1);
      }
    }
    else
    {
      *(_BYTE *)(v8 + a2) = v10;
      ++v8;
    }
    if ( v8 >= a3 )
      return v7;
  }
  return nullptr;
}

