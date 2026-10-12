/* ea:   0x000157A4 */
/* name: sub_157A4 */
/* size: 112 */
void *__fastcall sub_157A4(int a1, int a2, int a3, int a4)
{
  unsigned __int8 v5; // r5
  void *result; // r0

  v5 = a3;
  result = (void *)(a3 << 16);
  if ( (_WORD)a3 != 0 && (unsigned __int16)a3 <= 0xAu )
  {
    sub_16B48(a2 + 164, 18);
    sub_53FE4(a4 + 608, v5, 1);
    result = &dword_81184;
    if ( (_UNKNOWN *)(a1 + 36) != &dword_81184 )
      return (void *)sub_17890((int)&dword_81184);
  }
  return result;
}

