/* ea:   0x00045580 */
/* name: sub_45580 */
/* size: 44 */
void __fastcall sub_45580(int a1)
{
  void *v2; // r0

  v2 = *(void **)a1;
  if ( v2 != nullptr )
  {
    free(v2);
    *(_DWORD *)a1 = 0;
  }
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)(a1 + 8) = 0;
}

