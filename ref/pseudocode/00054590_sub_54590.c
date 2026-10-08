/* ea:   0x00054590 */
/* name: sub_54590 */
/* size: 108 */
int __fastcall sub_54590(int a1, int **a2)
{
  int v4; // r0
  int *v5; // r3
  int *v6; // r1
  int *v7; // r2
  char v9; // [sp+0h] [bp-10h] BYREF

  v4 = sub_545FC(a1, a2[1] - *a2, &v9);
  v5 = *a2;
  v6 = a2[1];
  v7 = *(int **)a1;
  if ( *a2 != v6 )
  {
    do
    {
      if ( v7 != nullptr )
        v4 = *v5;
      ++v5;
      if ( v7 != nullptr )
        *v7 = v4;
      ++v7;
    }
    while ( v5 != v6 );
  }
  *(_DWORD *)(a1 + 4) = v7;
  return a1;
}

