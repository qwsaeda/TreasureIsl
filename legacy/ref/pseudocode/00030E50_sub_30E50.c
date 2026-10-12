/* ea:   0x00030E50 */
/* name: sub_30E50 */
/* size: 20 */
void __fastcall sub_30E50(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

