/* ea:   0x0001586C */
/* name: sub_1586C */
/* size: 112 */
void *__fastcall sub_1586C(int a1, int a2, int a3, int a4)
{
  unsigned __int8 v5; // r5
  void *result; // r0

  v5 = a3;
  result = (void *)(a3 << 16);
  if ( (_WORD)a3 != 0 && (unsigned __int16)a3 <= 0xAu )
  {
    sub_16B48(a2 + 164, 19);
    sub_53FE4(a4 + 608, v5, 2);
    result = &dword_81184;
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      return (void *)sub_17890((int)&dword_81184);
  }
  return result;
}

