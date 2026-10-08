/* ea:   0x0004D35C */
/* name: sub_4D35C */
/* size: 36 */
int __fastcall sub_4D35C(int a1, char a2)
{
  int result; // r0

  if ( a2 == 0 )
    return sub_51F24(a1);
  result = *(_DWORD *)(a1 + 24);
  *(_BYTE *)(result + 49) = 1;
  return result;
}

