/* ea:   0x00049848 */
/* name: sub_49848 */
/* size: 92 */
int __fastcall sub_49848(int a1, char *a2)
{
  __int16 **v4; // r1
  char v5; // t1

  v5 = *a2;
  v4 = (__int16 **)(a2 + 4);
  *(_BYTE *)a1 = v5;
  if ( v4 != (__int16 **)(a1 + 4) )
    sub_17890(a1 + 4, *v4, v4[1]);
  *(_BYTE *)(a1 + 16) = a2[16];
  *(_DWORD *)(a1 + 20) = *((_DWORD *)a2 + 5);
  sub_4A57C(a1 + 24, a2 + 24);
  return a1;
}

