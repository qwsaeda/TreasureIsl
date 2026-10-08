/* ea:   0x00032478 */
/* name: sub_32478 */
/* size: 92 */
int __fastcall sub_32478(int a1, int a2, int a3)
{
  int v5; // r0
  void **v6; // r5
  int v7; // r6
  void **i; // r4

  v5 = sub_38080(a3, *(_DWORD *)(a1 + 4), a2);
  v6 = *(void ***)(a1 + 4);
  v7 = v5;
  for ( i = (void **)v5; i != v6; i += 3 )
    sub_12F48(i);
  *(_DWORD *)(a1 + 4) = v7;
  return a2;
}

