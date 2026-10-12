/* ea:   0x00048CDC */
/* name: sub_48CDC */
/* size: 36 */
int __fastcall sub_48CDC(int a1, char a2)
{
  int result; // r0

  if ( a2 == 0 )
    return sub_51F24(a1);
  result = *(_DWORD *)(a1 + 24);
  *(_BYTE *)(result + 48) = 1;
  return result;
}

