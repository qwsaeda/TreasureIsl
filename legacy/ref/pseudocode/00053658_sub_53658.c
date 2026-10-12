/* ea:   0x00053658 */
/* name: sub_53658 */
/* size: 156 */
int __fastcall sub_53658(int *a1, int a2, int a3)
{
  void *v4; // r0
  int v6; // r4

  *a1 = a2;
  a1[1] = a3;
  v4 = operator new(4 * a2);
  a1[2] = (int)v4;
  if ( v4 == nullptr )
    return 0;
  v6 = 0;
  if ( *a1 <= 0 )
    return 1;
  while ( 1 )
  {
    *(_DWORD *)(a1[2] + 4 * v6) = operator new(4 * a1[1]);
    memset(*(void **)(a1[2] + 4 * v6), 0, 4 * a1[1]);
    if ( *(_DWORD *)(a1[2] + 4 * v6) == 0 )
      break;
    if ( ++v6 >= *a1 )
      return 1;
  }
  return 0;
}

