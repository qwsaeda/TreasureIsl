/* ea:   0x00032960 */
/* name: sub_32960 */
/* size: 120 */
int __fastcall sub_32960(int a1, char a2)
{
  int v4; // r0
  char v6; // [sp+0h] [bp-10h] BYREF

  if ( sub_16C9C((unsigned __int16 *)(a1 + 136), 4) == 10 )
  {
    v4 = rand();
    sub_16B48((_DWORD *)(a1 + 164), *(unsigned __int8 *)(v4 % 10 + *(_DWORD *)(a1 + 164) + 1));
  }
  v6 = a2;
  return sub_161DC(a1 + 164, &v6);
}

