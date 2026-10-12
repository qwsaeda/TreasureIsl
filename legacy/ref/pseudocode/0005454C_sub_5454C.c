/* ea:   0x0005454C */
/* name: sub_5454C */
/* size: 68 */
int __fastcall sub_5454C(int a1, int a2)
{
  *(_BYTE *)a1 = *(_BYTE *)a2;
  sub_132FC((_DWORD *)(a1 + 4), (_DWORD *)(a2 + 4));
  *(_BYTE *)(a1 + 16) = *(_BYTE *)(a2 + 16);
  *(_DWORD *)(a1 + 20) = *(_DWORD *)(a2 + 20);
  sub_54590(a1 + 24, a2 + 24);
  return a1;
}

