/* ea:   0x0003B9D4 */
/* name: sub_3B9D4 */
/* size: 140 */
int __fastcall sub_3B9D4(int a1, int a2, int a3, int a4)
{
  int v7; // r4
  int v8; // r0
  unsigned int v9; // r2
  unsigned int v10; // r1

  v7 = rand() % 5 + 1;
  v8 = sub_3AA5C(a3, a4);
  v9 = *(_DWORD *)(v8 + 148);
  v10 = 1000 * v7 + *(_DWORD *)(v8 + 144);
  *(_DWORD *)(v8 + 144) = v10;
  if ( v10 > v9 )
    *(_DWORD *)(v8 + 144) = v9;
  return sub_3AB44(a1, 0, a3, a4, 1000 * v7);
}

