/* ea:   0x00030A30 */
/* name: sub_30A30 */
/* size: 20 */
void __fastcall sub_30A30(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

