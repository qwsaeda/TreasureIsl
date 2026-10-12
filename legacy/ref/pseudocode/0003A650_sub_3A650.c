/* ea:   0x0003A650 */
/* name: sub_3A650 */
/* size: 136 */
int __fastcall sub_3A650(_DWORD *a1)
{
  void *v2; // r0
  void *v3; // r0
  void *v4; // r0
  void *v5; // r0
  void *v6; // r0

  if ( a1[17] != 0 )
  {
    v2 = (void *)a1[13];
    if ( v2 == nullptr )
      __debugbreak();
    free(v2);
    a1[17] = 0;
  }
  v3 = (void *)a1[14];
  if ( v3 != nullptr )
  {
    free(v3);
    a1[14] = 0;
  }
  v4 = (void *)a1[15];
  if ( v4 != nullptr )
  {
    free(v4);
    a1[15] = 0;
  }
  v5 = (void *)a1[16];
  if ( v5 != nullptr )
  {
    free(v5);
    a1[16] = 0;
  }
  v6 = (void *)a1[17];
  if ( v6 != nullptr )
  {
    free(v6);
    a1[17] = 0;
  }
  return 1;
}

