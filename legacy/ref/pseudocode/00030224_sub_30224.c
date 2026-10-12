/* ea:   0x00030224 */
/* name: sub_30224 */
/* size: 20 */
void __fastcall sub_30224(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

